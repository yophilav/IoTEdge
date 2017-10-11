// Copyright (c) Microsoft. All rights reserved.

namespace Microsoft.Azure.Devices.Edge.Agent.IoTHub.Test
{
    using System;
    using System.Text;
    using Microsoft.Azure.Devices.Edge.Util.Test.Common;
    using Moq;
    using Xunit;

    public class ModuleIdentityTest
    {
        [Fact]
        [Unit]
        public void TestCreateInstance_ShouldThrowWhithNullArguments()
        {
            var connectionString = "fake";
            var moduleName = "module1";

            Assert.Throws<ArgumentNullException>(() => new ModuleIdentity(null, connectionString));
            Assert.Throws<ArgumentNullException>(() => new ModuleIdentity(moduleName, null));
        }

        [Fact]
        [Unit]
        public void TestEquals_ShouldReturnTrue()
        {
            var moduleName = "module1";
            var authMechanism = "fake";

            var m1 = new ModuleIdentity(moduleName, authMechanism);
            var m2 = new ModuleIdentity(moduleName, authMechanism);

            Assert.True(m1.Equals(m2));
            Assert.True(m2.Equals(m1));
        }

        [Fact]
        [Unit]
        public void TestEquals_SameReference_ShouldReturnTrue()
        {
            var moduleName = "module1";
            var authMechanism = "fake";

            var m1 = new ModuleIdentity(moduleName, authMechanism);
            var m2 = m1;

            Assert.True(m1.Equals(m2));
            Assert.True(m2.Equals(m1));
        }

        [Fact]
        [Unit]
        public void TestEquals_WithDifferentModuleId_ShouldReturnFalse()
        {
            var moduleName = "module";
            var authMechanism = "fake";

            var m1 = new ModuleIdentity(moduleName + "1", authMechanism);
            var m2 = new ModuleIdentity(moduleName + "2", authMechanism);

            Assert.False(m1.Equals(m2));
            Assert.False(m2.Equals(m1));
        }

    }
}
