// Copyright (c) Microsoft. All rights reserved.
namespace Microsoft.Azure.Devices.Edge.Test
{
    using System;
    using System.Threading;
    using System.Threading.Tasks;
    using Microsoft.Azure.Devices.Edge.Test.Common;
    using Microsoft.Azure.Devices.Edge.Test.Common.Certs;
    using Microsoft.Azure.Devices.Edge.Test.Common.Config;
    using Microsoft.Azure.Devices.Edge.Test.Helpers;
    using Microsoft.Azure.Devices.Edge.Util;
    using NUnit.Framework;

    class X509Device : X509ManualProvisioningFixture
    {
        [Test]
        public async Task X509QuickstartCerts()
        {
            CancellationToken token = this.TestToken;

            await this.runtime.DeployConfigurationAsync(token);

            string leafDeviceId =
                IdentityLimits.CheckLeafId($"{Context.Current.DeviceId}-x509-quickstart-certs");

            var leaf = await LeafDevice.CreateAsync(
                leafDeviceId,
                Protocol.Amqp,
                AuthenticationType.SelfSigned,
                Option.Some(Context.Current.DeviceId + "-x509"),
                false,
                CertificateAuthority.GetQuickstart(),
                this.iotHub,
                token);

            await TryFinally.DoAsync(
                async () =>
                {
                    DateTime seekTime = DateTime.Now;
                    await leaf.SendEventAsync(token);
                    await leaf.WaitForEventsReceivedAsync(seekTime, token);
                    await leaf.InvokeDirectMethodAsync(token);
                },
                async () =>
                {
                    await leaf.DeleteIdentityAsync(token);
                });
        }
    }
}
