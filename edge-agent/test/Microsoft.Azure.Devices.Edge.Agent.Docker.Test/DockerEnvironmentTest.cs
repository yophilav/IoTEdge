﻿// Copyright (c) Microsoft. All rights reserved.

namespace Microsoft.Azure.Devices.Edge.Agent.Docker.Test
{
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using System.Threading;
    using System.Threading.Tasks;
    using global::Docker.DotNet;
    using global::Docker.DotNet.Models;
    using Microsoft.Azure.Devices.Edge.Agent.Core;
    using Microsoft.Azure.Devices.Edge.Agent.Docker.Commands;
    using Microsoft.Azure.Devices.Edge.Util.Test.Common;
    using Microsoft.Extensions.Configuration;
    using Moq;
    using Xunit;

    [Collection("Docker")]
    public class DockerEnvironmentTest
    {
        static readonly TimeSpan Timeout = TimeSpan.FromSeconds(30);
        static readonly IDockerClient Client = DockerHelper.Client;

        [Fact]
        [Integration]
        public async Task TestEmptyEnvironment()
        {
            using (var cts = new CancellationTokenSource(Timeout))
            {
                var environment = new DockerEnvironment(Client);
                ModuleSet modules = await environment.GetModulesAsync(cts.Token);
                Assert.Equal(0, modules.Modules.Count);
            }
        }

        [Fact]
        [Integration]
        public async Task TestFilters()
        {
            const string Image = "hello-world";
            const string Tag = "latest";
            const string Name = "test-filters";
            const string FakeConnectionString = "FakeConnectionString";

            try
            {
                using (var cts = new CancellationTokenSource(Timeout))
                {
                    await Client.CleanupContainerAsync(Name, Image);
                    await Client.CleanupContainerAsync("test-filters-external", Image);

                    var loggingConfig = new DockerLoggingConfig("json-file");
                    var config = new DockerConfig(Image, Tag);
                    var module = new DockerModule(Name, "1.0", ModuleStatus.Running, config);

                    IConfigurationRoot configRoot = new ConfigurationBuilder().AddInMemoryCollection(new Dictionary<string, string>
                    {
                        { Constants.EdgeHubConnectionStringKey, FakeConnectionString }
                    }).Build();

                    var configSource = new Mock<IConfigSource>();
                    configSource.Setup(cs => cs.Configuration).Returns(configRoot);

                    var create = new CreateCommand(Client, module, loggingConfig, configSource.Object);

                    // pull the image for both containers
                    await Client.PullImageAsync(Image, Tag, cts.Token);

                    // pull and create module using commands
                    await create.ExecuteAsync(cts.Token);

                    var createParams = new CreateContainerParameters
                    {
                        Name = "test-filters-external",
                        Image = Image + ":" + Tag,
                    };
                    await Client.Containers.CreateContainerAsync(createParams);

                    // Check that only containers created via command are listed in the environment
                    var environment = new DockerEnvironment(Client);
                    ModuleSet modules = await environment.GetModulesAsync(cts.Token);
                    Assert.Equal(1, modules.Modules.Count);
                    Assert.Equal(module.Name, modules.Modules.First().Value.Name);
                }
            }
            finally
            {
                await Client.CleanupContainerAsync(Name, Image);
                await Client.CleanupContainerAsync("test-filters-external", Image);
            }
        }

        [Fact]
        [Integration]
        public async Task TestEnvVars()
        {
            const string Image = "hello-world";
            const string Tag = "latest";
            const string Name = "test-env";
            const string FakeConnectionString = "FakeConnectionString";

            try
            {
                using (var cts = new CancellationTokenSource(Timeout))
                {
                    await Client.CleanupContainerAsync(Name, Image);

                    var createOptions = @"{""Env"": [ ""k1=v1"", ""k2=v2""]}";
                    var config = new DockerConfig(Image, Tag, createOptions);
                    var loggingConfig = new DockerLoggingConfig("json-file");
                    var module = new DockerModule(Name, "1.0", ModuleStatus.Running, config);

                    IConfigurationRoot configRoot = new ConfigurationBuilder().AddInMemoryCollection(new Dictionary<string, string>
                    {
                        { Constants.EdgeHubConnectionStringKey, FakeConnectionString }
                    }).Build();

                    var configSource = new Mock<IConfigSource>();
                    configSource.Setup(cs => cs.Configuration).Returns(configRoot);

                    var create = new CreateCommand(Client, module, loggingConfig, configSource.Object);

                    await Client.PullImageAsync(Image, Tag, cts.Token);

                    // create module using command
                    await create.ExecuteAsync(cts.Token);

                    // check that the environment variables are being returned
                    var environment = new DockerEnvironment(Client);
                    ModuleSet modules = await environment.GetModulesAsync(cts.Token);
                    Assert.NotNull(modules.Modules[Name]);

                    Assert.True(((DockerEnvModule)modules.Modules[Name]).Config.CreateOptions.Env.Contains("k1=v1"));
                    Assert.True(((DockerEnvModule)modules.Modules[Name]).Config.CreateOptions.Env.Contains("k2=v2"));
                }
            }
            finally
            {
                await Client.CleanupContainerAsync(Name, Image);
                await Client.CleanupContainerAsync("test-filters-external", Image);
            }
        }

        [Fact]
        [Unit]
        public async Task ContainerToModuleTest()
        {
            const string StatusText = "Running for 1 second";
            const string LastStartText = "start time";
            const string LastExitText = "ending time";
            // Arrange
            var id = Guid.NewGuid().ToString();
            var containerListResponse = new ContainerListResponse
            {
                Image = "localhost:5000/sensor:v2",
                Names = new List<string> { "/sensor" },
                ID = id,
                State = "running",
                Labels = new Dictionary<string, string> { { "version", "v2" } }
            };
            var inspectContainerResponse = new ContainerInspectResponse
            {
                State = new ContainerState
                {
                    Status = StatusText,
                    ExitCode = 0,
                    StartedAt = LastStartText,
                    FinishedAt = LastExitText
                }
            };

            var dockerClient = Mock.Of<IDockerClient>(dc => 
                dc.Containers == Mock.Of<IContainerOperations>(co => co.InspectContainerAsync(id, default(CancellationToken)) == Task.FromResult(inspectContainerResponse)));

            // Act
            var dockerEnvironment = new DockerEnvironment(dockerClient);
            IModule module = await dockerEnvironment.ContainerToModule(containerListResponse);

            // Assert
            Assert.NotNull(module);
            var dockerModule = module as DockerEnvModule;
            Assert.NotNull(dockerModule);
            Assert.Equal("localhost:5000/sensor", dockerModule.Config.Image);
            Assert.Equal("v2", dockerModule.Config.Tag);
            Assert.Equal(0, dockerModule.Config.CreateOptions.Env?.Count ?? 0);
            Assert.Equal(0, dockerModule.Config.CreateOptions.HostConfig?.PortBindings?.Count ?? 0);

            Assert.Equal("sensor", dockerModule.Name);
            Assert.Equal("v2", dockerModule.Version);
            Assert.Equal(ModuleStatus.Running, dockerModule.Status);
            Assert.Equal(0, dockerModule.ExitCode);
            Assert.Equal(StatusText, dockerModule.StatusDescription);
            Assert.Equal(LastStartText, dockerModule.LastStartTime);
            Assert.Equal(LastExitText, dockerModule.LastExitTime);
        }
    }
}
