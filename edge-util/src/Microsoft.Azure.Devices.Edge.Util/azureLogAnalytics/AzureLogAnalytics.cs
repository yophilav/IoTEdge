// Copyright (c) Microsoft. All rights reserved.
namespace Microsoft.Azure.Devices.Edge.Util.AzureLogAnalytics
{
    using System;
    using System.Collections.Generic;
    using System.Net.Http;
    using System.Net.Http.Headers;
    using System.Security.Cryptography;
    using System.Text;
    using System.Threading.Tasks;
    using Microsoft.Extensions.Logging;
    using Newtonsoft.Json.Linq;

    public sealed class AzureLogAnalytics
    {
        static readonly ILogger Log = Logger.Factory.CreateLogger<AzureLogAnalytics>();
        static readonly AzureLogAnalytics instance = new AzureLogAnalytics();
        static readonly string apiVersion = "v1";
        static string accessToken = null;

        // Explicit static constructor to tell C# compiler
        // not to mark type as beforefieldinit
        static AzureLogAnalytics()
        {
        }

        AzureLogAnalytics()
        {
        }

        public static AzureLogAnalytics Instance
        {
            get
            {
                return instance;
            }
        }

        // Trigger Azure Active Directory (AAD) for an OAuth2 client credential for an azure resource access.
        // API reference: https://dev.loganalytics.io/documentation/Authorization/OAuth2
        public async Task<string> GetAccessToken(
            string azureActiveDirTenant,
            string azureActiveDirClientId,
            string azureActiveDirClientSecret,
            string azureResource)
        {
            try
            {
                Uri requestUri = new Uri($"https://login.microsoftonline.com/{azureActiveDirTenant}/oauth2/authorize");
                const string grantType = "client_credentials";

                var client = new HttpClient();
                client.DefaultRequestHeaders.Add("Content-Type", "application/x-www-form-urlencoded");

                var requestBody = new List<KeyValuePair<string, string>>();
                requestBody.Add(new KeyValuePair<string, string>("client_id", azureActiveDirClientId));
                requestBody.Add(new KeyValuePair<string, string>("client_secret", azureActiveDirClientSecret));
                requestBody.Add(new KeyValuePair<string, string>("grant_type", grantType));
                requestBody.Add(new KeyValuePair<string, string>("resource", azureResource));

                var request = new HttpRequestMessage(HttpMethod.Get, requestUri)
                {
                    Content = new FormUrlEncodedContent(requestBody)
                };

                var response = await client.SendAsync(request).ConfigureAwait(false);
                var responseMsg = await response.Content.ReadAsStringAsync().ConfigureAwait(false);
                Log.LogDebug(
                    ((int)response.StatusCode).ToString() + " " +
                    response.ReasonPhrase);

                return (string) JObject.Parse(responseMsg)["access_token"];
            }
            catch (Exception e)
            {
                Log.LogError(e.Message);
                throw e;
            }
        }
        // More info: 
        //    Requests: https://dev.loganalytics.io/documentation/Using-the-API/RequestFormat
        // public async GetQueryAsync(string workspaceId, string accessToken, string kqlQuery)
        // {
        //     Preconditions.CheckNotNull(workspaceId, "Log Analytic workspace ID cannot be empty.");
        //     Preconditions.CheckNotNull(accessToken, "Log Analytic access token cannot be empty.");
        //     // Preconditions.CheckNotNull(query, "Log Analytic query cannot be empty.");

        //     // TODO:
        //     // Check if we have Access token
        //     //    if not, request a new token, mark timestamp
        //     //    if yes,
        //     //      is it time out? ( expires_in is in seconds, 3600s = 1 hr)
        //     // Request the thing 




        //     try
        //     {
        //         string dateString = DateTime.UtcNow.ToString("r");
        //         Uri requestUri = new Uri($"https://api.loganalytics.io/{apiVersion}/workspaces/{workspaceId}/query");
        //         string signature = this.GetSignature("POST", query.Length, "application/json", dateString, "/api/logs", workspaceId, sharedKey);

        //         var client = new HttpClient();
        //         client.DefaultRequestHeaders.Add("Authorization", signature);
        //         client.DefaultRequestHeaders.Add("Accept", "application/json");
        //         client.DefaultRequestHeaders.Add("Log-Type", logType);
        //         client.DefaultRequestHeaders.Add("x-ms-date", dateString);

        //         var contentMsg = new StringContent(content, Encoding.UTF8);
        //         contentMsg.Headers.ContentType = new MediaTypeHeaderValue("application/json");
        //         Log.LogDebug(
        //             client.DefaultRequestHeaders.ToString() +
        //             contentMsg.Headers +
        //             contentMsg.ReadAsStringAsync().Result);

        //         var response = await client.PostAsync(requestUri, contentMsg).ConfigureAwait(false);
        //         var responseMsg = await response.Content.ReadAsStringAsync().ConfigureAwait(false);
        //         Log.LogDebug(
        //             ((int)response.StatusCode).ToString() + " " +
        //             response.ReasonPhrase + " " +
        //             responseMsg);
        //     }
        //     catch (Exception e)
        //     {
        //         Log.LogError(e.Message);
        //     }
        // }

        /* Sample code from:
        /* https://github.com/veyalla/MetricsCollector/blob/master/modules/MetricsCollector/AzureLogAnalytics.cs
        /* https://dejanstojanovic.net/aspnet/2018/february/send-data-to-azure-log-analytics-from-c-code/
        */
        public async void PostAsync(string workspaceId, string sharedKey, string content, string logType)
        {
            Preconditions.CheckNotNull(workspaceId, "Log Analytic workspace ID cannot be empty.");
            Preconditions.CheckNotNull(sharedKey, "Log Analytic shared key cannot be empty.");
            Preconditions.CheckNotNull(content, "Log Analytic content cannot be empty.");
            Preconditions.CheckNotNull(logType, "Log Analytic log type cannot be empty.");

            const string apiVersion = "2016-04-01";

            try
            {
                string dateString = DateTime.UtcNow.ToString("r");
                Uri requestUri = new Uri($"https://{workspaceId}.ods.opinsights.azure.com/api/logs?api-version={apiVersion}");
                string signature = this.GetSignature("POST", content.Length, "application/json", dateString, "/api/logs", workspaceId, sharedKey);

                var client = new HttpClient();
                client.DefaultRequestHeaders.Add("Authorization", signature);
                client.DefaultRequestHeaders.Add("Accept", "application/json");
                client.DefaultRequestHeaders.Add("Log-Type", logType);
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
                Log.LogError(e.Message);
            }
        }

        private string GetSignature(string method, int contentLength, string contentType, string date, string resource, string workspaceId, string sharedKey)
        {
            string message = $"{method}\n{contentLength}\n{contentType}\nx-ms-date:{date}\n{resource}";
            byte[] bytes = Encoding.UTF8.GetBytes(message);
            using (HMACSHA256 encryptor = new HMACSHA256(Convert.FromBase64String(sharedKey)))
            {
                return $"SharedKey {workspaceId}:{Convert.ToBase64String(encryptor.ComputeHash(bytes))}";
            }
        }
    }
}