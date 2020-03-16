// Copyright (c) Microsoft. All rights reserved.
namespace TestResultCoordinator.Reports.EdgeHubRestartTest
{
    using Microsoft.Azure.Devices.Edge.ModuleUtil;
    using Microsoft.Azure.Devices.Edge.Util;
    using TestResultCoordinator.Reports;

    class EdgeHubRestartDirectMethodReportMetadata : TestReportMetadataBase, ITestReportMetadata
    {
        public EdgeHubRestartDirectMethodReportMetadata(
            string testDescription,
            string senderSource,
            string receiverSource)
            : base(testDescription)
        {
            this.SenderSource = Preconditions.CheckNonWhiteSpace(senderSource, nameof(senderSource));
            this.ReceiverSource = Preconditions.CheckNonWhiteSpace(receiverSource, nameof(receiverSource));
        }

        public string SenderSource { get; }

        public string ReceiverSource { get; }

        public string[] ResultSources => new string[] { this.SenderSource, this.ReceiverSource };

        public override TestReportType TestReportType => TestReportType.EdgeHubRestartDirectMethodReport;

        public override TestOperationResultType TestOperationResultType => TestOperationResultType.EdgeHubRestartDirectMethod;
    }
}