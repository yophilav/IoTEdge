/*
 * Docker Engine API
 *
 * The Engine API is an HTTP API served by Docker Engine. It is the API the Docker client uses to communicate with the Engine, so everything the Docker client can do can be done with the API.  Most of the client's commands map directly to API endpoints (e.g. `docker ps` is `GET /containers/json`). The notable exception is running containers, which consists of several API calls.  # Errors  The API uses standard HTTP status codes to indicate the success or failure of the API call. The body of the response will be JSON in the following format:  ``` {   \"message\": \"page not found\" } ```  # Versioning  The API is usually changed in each release of Docker, so API calls are versioned to ensure that clients don't break.  For Docker Engine 17.10, the API version is 1.33. To lock to this version, you prefix the URL with `/v1.33`. For example, calling `/info` is the same as calling `/v1.33/info`.  Engine releases in the near future should support this version of the API, so your client will continue to work even if it is talking to a newer Engine.  In previous versions of Docker, it was possible to access the API without providing a version. This behaviour is now deprecated will be removed in a future version of Docker.  If the API version specified in the URL is not supported by the daemon, a HTTP `400 Bad Request` error message is returned.  The API uses an open schema model, which means server may add extra properties to responses. Likewise, the server will ignore any extra query parameters and request body properties. When you write clients, you need to ignore additional properties in responses to ensure they do not break when talking to newer Docker daemons.  This documentation is for version 1.34 of the API. Use this table to find documentation for previous versions of the API:  Docker version  | API version | Changes ----------------|-------------|--------- 17.10.x | [1.33](https://docs.docker.com/engine/api/v1.33/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-33-api-changes) 17.09.x | [1.32](https://docs.docker.com/engine/api/v1.32/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-32-api-changes) 17.07.x | [1.31](https://docs.docker.com/engine/api/v1.31/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-31-api-changes) 17.06.x | [1.30](https://docs.docker.com/engine/api/v1.30/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-30-api-changes) 17.05.x | [1.29](https://docs.docker.com/engine/api/v1.29/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-29-api-changes) 17.04.x | [1.28](https://docs.docker.com/engine/api/v1.28/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-28-api-changes) 17.03.1 | [1.27](https://docs.docker.com/engine/api/v1.27/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-27-api-changes) 1.13.1 & 17.03.0 | [1.26](https://docs.docker.com/engine/api/v1.26/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-26-api-changes) 1.13.0 | [1.25](https://docs.docker.com/engine/api/v1.25/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-25-api-changes) 1.12.x | [1.24](https://docs.docker.com/engine/api/v1.24/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-24-api-changes) 1.11.x | [1.23](https://docs.docker.com/engine/api/v1.23/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-23-api-changes) 1.10.x | [1.22](https://docs.docker.com/engine/api/v1.22/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-22-api-changes) 1.9.x | [1.21](https://docs.docker.com/engine/api/v1.21/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-21-api-changes) 1.8.x | [1.20](https://docs.docker.com/engine/api/v1.20/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-20-api-changes) 1.7.x | [1.19](https://docs.docker.com/engine/api/v1.19/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-19-api-changes) 1.6.x | [1.18](https://docs.docker.com/engine/api/v1.18/) | [API changes](https://docs.docker.com/engine/api/version-history/#v1-18-api-changes)  # Authentication  Authentication for registries is handled client side. The client has to send authentication details to various endpoints that need to communicate with registries, such as `POST /images/(name)/push`. These are sent as `X-Registry-Auth` header as a Base64 encoded (JSON) string with the following structure:  ``` {   \"username\": \"string\",   \"password\": \"string\",   \"email\": \"string\",   \"serveraddress\": \"string\" } ```  The `serveraddress` is a domain/IP without a protocol. Throughout this structure, double quotes are required.  If you have already got an identity token from the [`/auth` endpoint](#operation/SystemAuth), you can just pass this instead of credentials:  ``` {   \"identitytoken\": \"9cbaf023786cd7...\" } ```
 *
 * OpenAPI spec version: 1.34
 *
 * Generated by: https://github.com/swagger-api/swagger-codegen.git
 */

#[allow(unused_imports)]
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    ID: Option<String>,
    #[serde(rename = "Version", skip_serializing_if = "Option::is_none")]
    version: Option<::models::ObjectVersion>,
    /// Date and time at which the node was added to the swarm in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.
    #[serde(
        rename = "CreatedAt",
        skip_serializing_if = "Option::is_none"
    )]
    created_at: Option<String>,
    /// Date and time at which the node was last updated in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.
    #[serde(
        rename = "UpdatedAt",
        skip_serializing_if = "Option::is_none"
    )]
    updated_at: Option<String>,
    #[serde(rename = "Spec", skip_serializing_if = "Option::is_none")]
    spec: Option<::models::NodeSpec>,
    #[serde(
        rename = "Description",
        skip_serializing_if = "Option::is_none"
    )]
    description: Option<::models::NodeDescription>,
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    status: Option<::models::NodeStatus>,
    #[serde(
        rename = "ManagerStatus",
        skip_serializing_if = "Option::is_none"
    )]
    manager_status: Option<::models::ManagerStatus>,
}

impl Node {
    pub fn new() -> Node {
        Node {
            ID: None,
            version: None,
            created_at: None,
            updated_at: None,
            spec: None,
            description: None,
            status: None,
            manager_status: None,
        }
    }

    pub fn set_ID(&mut self, ID: String) {
        self.ID = Some(ID);
    }

    pub fn with_ID(mut self, ID: String) -> Node {
        self.ID = Some(ID);
        self
    }

    pub fn ID(&self) -> Option<&String> {
        self.ID.as_ref()
    }

    pub fn reset_ID(&mut self) {
        self.ID = None;
    }

    pub fn set_version(&mut self, version: ::models::ObjectVersion) {
        self.version = Some(version);
    }

    pub fn with_version(mut self, version: ::models::ObjectVersion) -> Node {
        self.version = Some(version);
        self
    }

    pub fn version(&self) -> Option<&::models::ObjectVersion> {
        self.version.as_ref()
    }

    pub fn reset_version(&mut self) {
        self.version = None;
    }

    pub fn set_created_at(&mut self, created_at: String) {
        self.created_at = Some(created_at);
    }

    pub fn with_created_at(mut self, created_at: String) -> Node {
        self.created_at = Some(created_at);
        self
    }

    pub fn created_at(&self) -> Option<&String> {
        self.created_at.as_ref()
    }

    pub fn reset_created_at(&mut self) {
        self.created_at = None;
    }

    pub fn set_updated_at(&mut self, updated_at: String) {
        self.updated_at = Some(updated_at);
    }

    pub fn with_updated_at(mut self, updated_at: String) -> Node {
        self.updated_at = Some(updated_at);
        self
    }

    pub fn updated_at(&self) -> Option<&String> {
        self.updated_at.as_ref()
    }

    pub fn reset_updated_at(&mut self) {
        self.updated_at = None;
    }

    pub fn set_spec(&mut self, spec: ::models::NodeSpec) {
        self.spec = Some(spec);
    }

    pub fn with_spec(mut self, spec: ::models::NodeSpec) -> Node {
        self.spec = Some(spec);
        self
    }

    pub fn spec(&self) -> Option<&::models::NodeSpec> {
        self.spec.as_ref()
    }

    pub fn reset_spec(&mut self) {
        self.spec = None;
    }

    pub fn set_description(&mut self, description: ::models::NodeDescription) {
        self.description = Some(description);
    }

    pub fn with_description(mut self, description: ::models::NodeDescription) -> Node {
        self.description = Some(description);
        self
    }

    pub fn description(&self) -> Option<&::models::NodeDescription> {
        self.description.as_ref()
    }

    pub fn reset_description(&mut self) {
        self.description = None;
    }

    pub fn set_status(&mut self, status: ::models::NodeStatus) {
        self.status = Some(status);
    }

    pub fn with_status(mut self, status: ::models::NodeStatus) -> Node {
        self.status = Some(status);
        self
    }

    pub fn status(&self) -> Option<&::models::NodeStatus> {
        self.status.as_ref()
    }

    pub fn reset_status(&mut self) {
        self.status = None;
    }

    pub fn set_manager_status(&mut self, manager_status: ::models::ManagerStatus) {
        self.manager_status = Some(manager_status);
    }

    pub fn with_manager_status(mut self, manager_status: ::models::ManagerStatus) -> Node {
        self.manager_status = Some(manager_status);
        self
    }

    pub fn manager_status(&self) -> Option<&::models::ManagerStatus> {
        self.manager_status.as_ref()
    }

    pub fn reset_manager_status(&mut self) {
        self.manager_status = None;
    }
}
