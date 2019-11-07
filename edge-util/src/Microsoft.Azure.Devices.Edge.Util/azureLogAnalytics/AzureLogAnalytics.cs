// Copyright (c) Microsoft. All rights reserved.
namespace Microsoft.Azure.Devices.Edge.Util.AzureLogAnalytics
{
    using System;
    using System.IO;
    using System.Net;
    using System.Net.Http;
    using System.Net.Http.Headers;
    using System.Security.Cryptography;
    using System.Text;
    using Microsoft.Extensions.Logging;
    using Newtonsoft.Json;

    /* Sample code from:
    /* https://github.com/veyalla/MetricsCollector/blob/master/modules/MetricsCollector/AzureLogAnalytics.cs
    /* https://dejanstojanovic.net/aspnet/2018/february/send-data-to-azure-log-analytics-from-c-code/
    */

    public sealed class AzureLogAnalytics
    {
        static readonly ILogger Log = Logger.Factory.CreateLogger<AzureLogAnalytics>();
        static AzureLogAnalytics instance = null;
        public string WorkspaceId { get; }
        public string SharedKey { get; }
        public string ApiVersion { get; }
        AzureLogAnalytics(string workspaceId, string sharedKey)
        {
            const string apiVersion = "2016-04-01";

            Preconditions.CheckNotNull(workspaceId, "Log Analytic workspace ID cannot be empty.");
            Preconditions.CheckNotNull(sharedKey, "Log Analytic shared key cannot be empty.");

            this.WorkspaceId = workspaceId;
            this.SharedKey = sharedKey;
            this.ApiVersion = apiVersion;
        }

        public static AzureLogAnalytics GetInstance()
        {
            Preconditions.CheckNotNull(instance);
            return instance;
        }

        public static AzureLogAnalytics InitInstance(string workspaceId, string sharedKey)
        {
            if (instance == null)
            {
                instance = new AzureLogAnalytics(workspaceId, sharedKey);
            }

            return instance;
        }

        public async void PostAsync(string content, string LogType)
        {
            try
            {
                string dateString = DateTime.UtcNow.ToString("r");
                Uri requestUri = new Uri($"https://{this.WorkspaceId}.ods.opinsights.azure.com/api/logs?api-version={this.ApiVersion}");
                string signature = this.GetSignature("POST", content.Length, "application/json", dateString, "/api/logs");

                var client = new HttpClient();
                client.DefaultRequestHeaders.Add("Authorization", signature);
                client.DefaultRequestHeaders.Add("Accept", "application/json");
                client.DefaultRequestHeaders.Add("Log-Type", LogType);
                client.DefaultRequestHeaders.Add("x-ms-date", dateString);

                var contentMsg = new StringContent(content, Encoding.UTF8);
                contentMsg.Headers.ContentType = new MediaTypeHeaderValue("application/json");
                Log.LogDebug(
                    client.DefaultRequestHeaders.ToString() +
                    contentMsg.Headers +
                    contentMsg.ReadAsStringAsync().Result);

                var response = await client.PostAsync(requestUri, contentMsg).ConfigureAwait(false);
                var responseMsg = await response.Content.ReadAsStringAsync().ConfigureAwait(false);
                Log.LogDebug(
                    ((int)response.StatusCode).ToString() + " " +
                    response.ReasonPhrase + " " +
                    responseMsg);
            }
            catch (Exception e)
            {
                Log.LogDebug(e.Message);
                Log.LogError(e.Message);
            }
        }

        private string GetSignature(string method, int contentLength, string contentType, string date, string resource)
        {
            string message = $"{method}\n{contentLength}\n{contentType}\nx-ms-date:{date}\n{resource}";
            byte[] bytes = Encoding.UTF8.GetBytes(message);
            using (HMACSHA256 encryptor = new HMACSHA256(Convert.FromBase64String(this.SharedKey)))
            {
                return $"SharedKey {this.WorkspaceId}:{Convert.ToBase64String(encryptor.ComputeHash(bytes))}";
            }
        }
    }
}