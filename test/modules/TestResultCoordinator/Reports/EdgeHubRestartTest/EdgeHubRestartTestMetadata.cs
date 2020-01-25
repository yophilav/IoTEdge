// Copyright (c) Microsoft. All rights reserved.
namespace TestResultCoordinator.Reports.EdgeHubRestartTest
{
    using System;
    using System.Linq;
    using Microsoft.Azure.Devices.Edge.ModuleUtil;
    using Microsoft.Azure.Devices.Edge.Util;
    using TestResultCoordinator.Reports;

    class EdgeHubRestartTestMetadata : ITestReportMetadata
    {
        public EdgeHubRestartTestMetadata(
            string senderSource,
            string restarterSource,
            TimeSpan? tolerancePeriod = null,
            string receiverSource = "")
        {
            this.SenderSource = Preconditions.CheckNonWhiteSpace(senderSource, nameof(senderSource));;
            this.RestarterSource = Preconditions.CheckNonWhiteSpace(restarterSource, nameof(restarterSource));
            this.TolerancePeriod = tolerancePeriod ?? new TimeSpan(0, 0, 0, 0, 5);
            this.ReceiverSource = string.IsNullOrEmpty(receiverSource) ? Option.None<string>() : Option.Some(receiverSource);
        }
        public string SenderSource { get; }

        public string RestarterSource { get; }

        public Option<string> ReceiverSource { get; }

        public string[] ResultSources =>
            this.ReceiverSource.HasValue ? new string[] { this.SenderSource, this.ReceiverSource.OrDefault() } : new string[] { this.SenderSource };

        public TimeSpan TolerancePeriod { get; }

        public TestReportType TestReportType => TestReportType.EdgeHubRestartReport;

        public TestOperationResultType TestOperationResultType => (TestOperationResultType)Enum.Parse(typeof(TestOperationResultType), SenderSource.Split('.').LastOrDefault()) ;
    }
}
