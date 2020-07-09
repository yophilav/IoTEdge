// Copyright (c) Microsoft. All rights reserved.
namespace IotEdgeQuickstart.Details
{
    using System;
    using System.ComponentModel;
    using System.Diagnostics;
    using System.Threading;
    using System.Threading.Tasks;
    using RunProcessAsTask;
    using Serilog;

    public class Process
    {
        public static async Task<string[]> RunAsync(string name, string args, int timeoutSeconds = 15)
        {
            using (var cts = new CancellationTokenSource(TimeSpan.FromSeconds(timeoutSeconds)))
            {
                return await RunAsync(name, args, cts.Token);
            }
        }

        public static async Task<string[]> RunAsync(string name, string args, CancellationToken token)
        {
            var info = new ProcessStartInfo
            {
                FileName = name,
                Arguments = args,
            };

            using (ProcessResults result = await ProcessEx.RunAsync(info, token))
            {
                if (result.ExitCode != 0)
                {
                    throw new Win32Exception(result.ExitCode, $"{string.Join("\n", result.StandardOutput)}\n\n'{name}' failed with: {string.Join("\n", result.StandardError)}");
                }

                return result.StandardOutput;
            }
        }
    }
}
