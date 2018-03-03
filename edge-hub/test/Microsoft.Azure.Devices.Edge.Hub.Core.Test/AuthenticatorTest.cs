// Copyright (c) Microsoft. All rights reserved.
namespace Microsoft.Azure.Devices.Edge.Hub.Core.Test
{
    using System;
    using System.Threading.Tasks;
    using Microsoft.Azure.Devices.Edge.Hub.Core;
    using Microsoft.Azure.Devices.Edge.Hub.Core.Cloud;
    using Microsoft.Azure.Devices.Edge.Hub.Core.Device;
    using Microsoft.Azure.Devices.Edge.Hub.Core.Identity;
    using Microsoft.Azure.Devices.Edge.Util;
    using Microsoft.Azure.Devices.Edge.Util.Test.Common;
    using Moq;
    using Xunit;

    public class AuthenticatorTest
    {
        [Fact]
        [Unit]
        public void AuthenticatorConstructorTest()
        {
            var connectionManager = Mock.Of<IConnectionManager>();
            Assert.NotNull(new Authenticator(connectionManager, "your-device"));
        }

        [Fact]
        [Unit]
        public void AuthenticatorConstructor_NullConnectionManagerTest()
        {
            Assert.Throws<ArgumentNullException>(() => new Authenticator(null, "your-device"));
        }

        [Fact]
        [Unit]
        public void AuthenticatorConstructor_NullDeviceIdTest()
        {
            var connectionManager = Mock.Of<IConnectionManager>();
            Assert.Throws<ArgumentException>(() => new Authenticator(connectionManager, null));
        }

        [Fact]
        [Unit]
        public void AuthenticatorConstructor_EmptyDeviceIdTest()
        {
            var connectionManager = Mock.Of<IConnectionManager>();
            Assert.Throws<ArgumentException>(() => new Authenticator(connectionManager, string.Empty));
        }

        [Fact]
        [Unit]
        public async Task AuthenticateTest()
        {
            var cloudProxy = Mock.Of<ICloudProxy>();
            var connectionManager = Mock.Of<IConnectionManager>();
            var identity = Mock.Of<IIdentity>();

            Mock.Get(connectionManager).Setup(cm => cm.CreateCloudConnectionAsync(identity)).ReturnsAsync(Try.Success(cloudProxy));
            Mock.Get(cloudProxy).Setup(cp => cp.IsActive).Returns(true);

            var authenticator = new Authenticator(connectionManager, "your-device");
            Assert.Equal(true, await authenticator.AuthenticateAsync(identity));
        }

        [Fact]
        [Unit]
        public async Task Authenticate_InactiveProxyTest()
        {
            var cloudProxy = Mock.Of<ICloudProxy>();
            var connectionManager = Mock.Of<IConnectionManager>();
            var identity = Mock.Of<IIdentity>();

            Mock.Get(connectionManager).Setup(cm => cm.CreateCloudConnectionAsync(identity)).ReturnsAsync(Try.Success(cloudProxy));
            Mock.Get(cloudProxy).Setup(cp => cp.IsActive).Returns(false);

            var authenticator = new Authenticator(connectionManager, "your-device");
            Assert.Equal(false, await authenticator.AuthenticateAsync(identity));
        }

        [Fact]
        [Unit]
        public async Task Authenticate_ConnectionManagerThrowsTest()
        {
            var cloudProxy = Mock.Of<ICloudProxy>();
            var connectionManager = Mock.Of<IConnectionManager>();
            var identity = Mock.Of<IIdentity>();

            Mock.Get(connectionManager).Setup(cm => cm.CreateCloudConnectionAsync(identity)).ReturnsAsync(Try<ICloudProxy>.Failure(new ArgumentException()));
            Mock.Get(cloudProxy).Setup(cp => cp.IsActive).Returns(true);

            var authenticator = new Authenticator(connectionManager, "your-device");
            Assert.Equal(false, await authenticator.AuthenticateAsync(identity));
        }

        [Fact]
        [Unit]
        public async Task Authenticate_NonNullIdentityTest ()
        {
            var connectionManager = Mock.Of<IConnectionManager>();
            var moduleIdentity = Mock.Of<IModuleIdentity>();

            Mock.Get(moduleIdentity).Setup(mi => mi.DeviceId).Returns("my-device");

            var authenticator = new Authenticator(connectionManager, "your-device");
            Assert.Equal(false, await authenticator.AuthenticateAsync(moduleIdentity));
        }

        [Fact]
        [Unit]
        public async Task Authenticate_NullIdentityTest()
        {
            var connectionManager = Mock.Of<IConnectionManager>();

            var authenticator = new Authenticator(connectionManager, "your-device");
            await Assert.ThrowsAsync<ArgumentNullException>(() => authenticator.AuthenticateAsync(null));
        }

        [Fact]
        [Unit]
        public async Task Authenticate_x509Identity()
        {
            var connectionManager = Mock.Of<IConnectionManager>();
            var moduleIdentity = Mock.Of<IModuleIdentity>();

            Mock.Get(moduleIdentity).Setup(mi => mi.DeviceId).Returns("my-device");
            Mock.Get(moduleIdentity).Setup(mi => mi.Scope).Returns(AuthenticationScope.x509Cert);

            var authenticator = new Authenticator(connectionManager, "my-device");
            Assert.Equal(true, await authenticator.AuthenticateAsync(moduleIdentity));
        }
    }
}
