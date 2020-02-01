// Copyright (c) Microsoft. All rights reserved.
namespace EdgeHubRestartTester
{
    using System;
    using System.Net;
    using System.Net.Sockets;
    using System.Text;
    using System.Text.RegularExpressions;
    using System.Threading;
    using System.Threading.Tasks;
    using Microsoft.Azure.Devices.Client;
    using Microsoft.Azure.Devices.Client.Exceptions;
    using Microsoft.Azure.Devices.Edge.ModuleUtil;
    using Microsoft.Azure.Devices.Edge.ModuleUtil.TestResults;
    using Microsoft.Azure.Devices.Edge.Util;
    using Microsoft.Extensions.Logging;

    class DirectMethodEdgeHubConnector : IEdgeHubConnector
    {
        long directMethodCount = 0;
        ModuleClient dmModuleClient;
        Guid batchId;
        DateTime runExpirationTime;
        CancellationToken cancellationToken;
        DateTime edgeHubRestartedTime;
        HttpStatusCode edgeHubRestartStatusCode;
        uint restartSequenceNumber;
        ILogger logger;

        public DirectMethodEdgeHubConnector(
            ModuleClient dmModuleClient,
            Guid batchId,
            DateTime runExpirationTime,
            DateTime edgeHubRestartedTime,
            HttpStatusCode edgeHubRestartStatusCode,
            uint restartSequenceNumber,
            ILogger logger,
            CancellationToken cancellationToken)
        {
            this.dmModuleClient = Preconditions.CheckNotNull(dmModuleClient, nameof(dmModuleClient));
            this.batchId = batchId;
            this.runExpirationTime = runExpirationTime;
            this.cancellationToken = Preconditions.CheckNotNull(cancellationToken, nameof(cancellationToken));
            this.edgeHubRestartedTime = edgeHubRestartedTime;
            this.edgeHubRestartStatusCode = edgeHubRestartStatusCode;
            this.restartSequenceNumber = restartSequenceNumber;
            this.logger = Preconditions.CheckNotNull(logger, nameof(logger));
        }

        public async Task StartAsync()
        {
            (DateTime dmCompletedTime, HttpStatusCode dmStatusCode) = await this.SendDirectMethodAsync(
                Settings.Current.DeviceId,
                Settings.Current.DirectMethodTargetModuleId,
                this.dmModuleClient,
                Settings.Current.DirectMethodName,
                this.runExpirationTime,
                this.cancellationToken,
                this.logger).ConfigureAwait(false);

            TestResultBase dmTestResult = new EdgeHubRestartDirectMethodResult(
                Settings.Current.ModuleId + "." + TestOperationResultType.EdgeHubRestartDirectMethod.ToString(),
                DateTime.UtcNow,
                Settings.Current.TrackingId,
                this.batchId,
                Interlocked.Read(ref this.directMethodCount).ToString(),
                this.edgeHubRestartedTime,
                this.edgeHubRestartStatusCode,
                dmCompletedTime,
                dmStatusCode,
                this.restartSequenceNumber);

            var reportClient = new TestResultReportingClient { BaseUrl = Settings.Current.ReportingEndpointUrl.AbsoluteUri };
            await ModuleUtil.ReportTestResultUntilSuccessAsync(
                reportClient,
                this.logger,
                dmTestResult,
                this.cancellationToken).ConfigureAwait(false);
        }

        async Task<Tuple<DateTime, HttpStatusCode>> SendDirectMethodAsync(
            string deviceId,
            string targetModuleId,
            ModuleClient moduleClient,
            string directMethodName,
            DateTime runExpirationTime,
            CancellationToken cancellationToken,
            ILogger logger)
        {
            while ((!cancellationToken.IsCancellationRequested) && (DateTime.UtcNow < runExpirationTime))
            {
                try
                {
                    // Direct Method sequence number is always increasing regardless of sending result.
                    Interlocked.Increment(ref this.directMethodCount);
                    MethodRequest request = new MethodRequest(
                        directMethodName,
                        Encoding.UTF8.GetBytes($"{{ \"Message\": \"Hello\", \"DirectMethodCount\": \"{Interlocked.Read(ref this.directMethodCount).ToString()}\" }}"));
                    MethodResponse result = await moduleClient.InvokeMethodAsync(deviceId, targetModuleId, request);
                    if ((HttpStatusCode)result.Status == HttpStatusCode.OK)
                    {
                        logger.LogDebug(result.ResultAsJson);
                    }
                    else
                    {
                        logger.LogError(result.ResultAsJson);
                    }

                    logger.LogInformation($"[DirectMethodEdgeHubConnector] Invoke DirectMethod with count {Interlocked.Read(ref this.directMethodCount).ToString()}");
                    return new Tuple<DateTime, HttpStatusCode>(DateTime.UtcNow, (HttpStatusCode)result.Status);
                }
                catch (IotHubCommunicationException e)
                {
                    // Only handle the exception that relevant to our test case; otherwise, re-throw it.
                    if (this.IsEdgeHubDownDuringDirectMethodSend(e))
                    {
                        // swallow exeception and retry until success
                        logger.LogDebug(e, $"[DirectMethodEdgeHubConnector] Exception caught with SequenceNumber {Interlocked.Read(ref this.directMethodCount).ToString()}");
                    }
                    else
                    {
                        // something is wrong, Log and re-throw
                        logger.LogError(e, $"[DirectMethodEdgeHubConnector] Exception caught with SequenceNumber {Interlocked.Read(ref this.directMethodCount).ToString()}");
                        throw;
                    }
                }
                catch (DeviceNotFoundException e)
                {
                    if (this.IsDirectMethodReceiverNotConnected(e))
                    {
                        // swallow exeception and retry until success
                        logger.LogDebug(e, $"[DirectMethodEdgeHubConnector] Exception caught with SequenceNumber {Interlocked.Read(ref this.directMethodCount).ToString()}");
                    }
                    else
                    {
                        // something is wrong, Log and re-throw
                        logger.LogError(e, $"[DirectMethodEdgeHubConnector] Exception caught with SequenceNumber {Interlocked.Read(ref this.directMethodCount).ToString()}");
                        throw;
                    }
                }
            }

            return new Tuple<DateTime, HttpStatusCode>(DateTime.UtcNow, HttpStatusCode.InternalServerError);
        }

        bool IsEdgeHubDownDuringDirectMethodSend(IotHubCommunicationException e)
        {
            // This is a socket exception error code when EdgeHub is down.
            const int EdgeHubNotAvailableErrorCode = 111;

            if (e?.InnerException?.InnerException is SocketException)
            {
                int errorCode = ((SocketException)e.InnerException.InnerException).ErrorCode;
                return errorCode == EdgeHubNotAvailableErrorCode;
            }

            return false;
        }

        bool IsDirectMethodReceiverNotConnected(DeviceNotFoundException e)
        {
            string errorMsg = e.Message;
            return Regex.IsMatch(errorMsg, $"\\b{Settings.Current.DirectMethodTargetModuleId}\\b");
        }
    }
}