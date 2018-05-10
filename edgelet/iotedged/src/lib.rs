// Copyright (c) Microsoft. All rights reserved.

#![deny(warnings)]

extern crate base64;
#[macro_use]
extern crate clap;
extern crate config;
extern crate edgelet_core;
extern crate edgelet_docker;
extern crate edgelet_hsm;
extern crate edgelet_http;
extern crate edgelet_http_mgmt;
extern crate edgelet_http_workload;
extern crate edgelet_iothub;
extern crate env_logger;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate iothubservice;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_signal;
extern crate url;
extern crate url_serde;

pub mod app;
mod error;
pub mod logging;
pub mod settings;
pub mod signal;

use edgelet_core::ModuleSpec;
use edgelet_core::crypto::{DerivedKeyStore, KeyStore, MemoryKey};
use edgelet_core::provisioning::{ManualProvisioning, Provision};
use edgelet_core::watchdog::Watchdog;
use edgelet_docker::{DockerConfig, DockerModuleRuntime};
use edgelet_hsm::Crypto;
use edgelet_http::client::Client as HttpClient;
use edgelet_http::logging::LoggingService;
use edgelet_http::{ApiVersionService, HyperExt, Run, API_VERSION};
use edgelet_http_mgmt::ManagementService;
use edgelet_http_workload::WorkloadService;
use edgelet_iothub::{HubIdentityManager, SasTokenSource};
use futures::Future;
use futures::sync::oneshot::{self, Receiver};
use hyper::Client as HyperClient;
use hyper::client::HttpConnector;
use hyper::server::Http;
use hyper_tls::HttpsConnector;
use iothubservice::DeviceClient;
use std::collections::HashMap;
use tokio_core::reactor::{Core, Handle};
use url::Url;

use settings::{Provisioning, Settings};

pub use self::error::{Error, ErrorKind};

const EDGE_RUNTIME_MODULEID: &str = "$edgeAgent";
const EDGE_RUNTIME_MODULE_NAME: &str = "edgeAgent";
const AUTH_SCHEME: &str = "sasToken";
const HOSTNAME_KEY: &str = "IOTEDGE_IOTHUBHOSTNAME";
const GATEWAY_HOSTNAME_KEY: &str = "EDGEDEVICEHOSTNAME";
const DEVICEID_KEY: &str = "IOTEDGE_DEVICEID";
const MODULEID_KEY: &str = "IOTEDGE_MODULEID";
const WORKLOAD_URI_KEY: &str = "IOTEDGE_IOTEDGEDURI";
const VERSION_KEY: &str = "IOTEDGE_IOTEDGEDVERSION";
const AUTHSCHEME_KEY: &str = "IOTEDGE_AUTHSCHEME";
const MANAGEMENT_URI_KEY: &str = "MANAGEMENTURI";
const IOTHUB_API_VERSION: &str = "2017-11-08-preview";
const DNS_WORKER_THREADS: usize = 4;

pub struct Main {
    settings: Settings<DockerConfig>,
    reactor: Core,
}

impl Main {
    pub fn new(settings: Settings<DockerConfig>) -> Result<Self, Error> {
        let reactor = Core::new()?;
        let main = Main { settings, reactor };
        Ok(main)
    }

    pub fn handle(&self) -> Handle {
        self.reactor.handle()
    }

    pub fn run_until<F>(self, shutdown_signal: F) -> Result<(), Error>
    where
        F: Future<Item = (), Error = ()> + 'static,
    {
        let Main {
            settings,
            reactor: mut core,
        } = self;

        let provisioning_settings = settings.provisioning();
        let (key_store, hub_name, device_id, root_key) = provision(provisioning_settings)?;

        info!(
            "Manual provisioning with DeviceId({}) and HostName({})",
            device_id, hub_name
        );

        let handle: Handle = core.handle().clone();

        let runtime = DockerModuleRuntime::new(settings.docker_uri(), &handle)?;
        let hostname = format!("https://{}", hub_name);

        let hyper_client = HyperClient::configure()
            .connector(HttpsConnector::new(DNS_WORKER_THREADS, &handle)?)
            .build(&handle);
        let http_client = HttpClient::new(
            hyper_client,
            Some(SasTokenSource::new(
                hub_name.clone(),
                device_id.clone(),
                root_key,
            )),
            IOTHUB_API_VERSION,
            Url::parse(&hostname)?,
        )?;
        let device_client = DeviceClient::new(http_client, &device_id)?;
        let id_man = HubIdentityManager::new(key_store.clone(), device_client);

        let (mgmt_tx, mgmt_rx) = oneshot::channel();
        let (work_tx, work_rx) = oneshot::channel();

        let mgmt = start_management(
            settings.management_uri().clone(),
            &core.handle(),
            &runtime,
            &id_man,
            mgmt_rx,
        )?;

        let workload = start_workload(
            settings.workload_uri().clone(),
            &key_store,
            &core.handle(),
            work_rx,
        )?;

        start_runtime(
            &runtime,
            &id_man,
            &mut core,
            &hub_name,
            &device_id,
            &settings,
        )?;

        let shutdown = shutdown_signal.map(move |_| {
            debug!("shutdown signaled");
            mgmt_tx.send(()).unwrap_or(());
            work_tx.send(()).unwrap_or(());
        });

        core.handle().spawn(shutdown);

        core.run(mgmt.join(workload))?;
        info!("Shutdown complete");
        Ok(())
    }
}

fn provision(
    provisioning: &Provisioning,
) -> Result<(DerivedKeyStore<MemoryKey>, String, String, MemoryKey), Error> {
    match *provisioning {
        Provisioning::Manual {
            ref device_connection_string,
        } => {
            let provision = ManualProvisioning::new(device_connection_string.as_str())?;
            let root_key = MemoryKey::new(base64::decode(provision.key()?)?);
            let key_store = DerivedKeyStore::new(root_key.clone());
            let hub_name = provision.host_name().to_string();
            let device_id = provision.device_id().to_string();
            Ok((key_store, hub_name, device_id, root_key))
        }
        _ => unimplemented!(),
    }
}

fn start_runtime(
    runtime: &DockerModuleRuntime,
    id_man: &HubIdentityManager<
        DerivedKeyStore<MemoryKey>,
        HyperClient<HttpsConnector<HttpConnector>>,
    >,
    core: &mut Core,
    hostname: &str,
    device_id: &str,
    settings: &Settings<DockerConfig>,
) -> Result<(), Error> {
    let spec = settings.runtime().clone();
    let env = build_env(spec.env(), hostname, device_id, settings);
    let spec = ModuleSpec::<DockerConfig>::new(
        EDGE_RUNTIME_MODULE_NAME,
        spec.type_(),
        spec.config().clone(),
        env,
    )?;
    let mut watchdog = Watchdog::new(runtime.clone(), id_man.clone());
    let runtime_future = watchdog.start(spec, EDGE_RUNTIME_MODULEID);
    // TODO: When this is converted to a watchdog that keeps running, convert this to use a handle
    // that allows it to shutdown gracefully.
    core.run(runtime_future)?;
    Ok(())
}

// Add the environment variables needed by the EdgeAgent.
fn build_env(
    spec_env: &HashMap<String, String>,
    hostname: &str,
    device_id: &str,
    settings: &Settings<DockerConfig>,
) -> HashMap<String, String> {
    let workload_uri = format!(
        "http://{}:{}",
        settings.hostname(),
        settings
            .workload_uri()
            .port_or_known_default()
            .unwrap_or(80),
    );

    let management_uri = format!(
        "http://{}:{}",
        settings.hostname(),
        settings
            .management_uri()
            .port_or_known_default()
            .unwrap_or(80),
    );

    let mut env = HashMap::new();
    env.insert(HOSTNAME_KEY.to_string(), hostname.to_string());
    env.insert(
        GATEWAY_HOSTNAME_KEY.to_string(),
        settings.hostname().to_string(),
    );
    env.insert(DEVICEID_KEY.to_string(), device_id.to_string());
    env.insert(MODULEID_KEY.to_string(), EDGE_RUNTIME_MODULEID.to_string());
    env.insert(WORKLOAD_URI_KEY.to_string(), workload_uri);
    env.insert(VERSION_KEY.to_string(), API_VERSION.to_string());
    env.insert(AUTHSCHEME_KEY.to_string(), AUTH_SCHEME.to_string());
    env.insert(MANAGEMENT_URI_KEY.to_string(), management_uri);

    for (key, val) in spec_env.iter() {
        env.insert(key.clone(), val.clone());
    }
    env
}

fn start_management(
    addr: Url,
    handle: &Handle,
    mgmt: &DockerModuleRuntime,
    id_man: &HubIdentityManager<
        DerivedKeyStore<MemoryKey>,
        HyperClient<HttpsConnector<HttpConnector>>,
    >,
    shutdown: Receiver<()>,
) -> Result<Run, Error> {
    let server_handle = handle.clone();
    let service = LoggingService::new(ApiVersionService::new(ManagementService::new(
        mgmt,
        id_man,
    )?));

    info!("Listening on {} with 1 thread for management API.", addr);

    let run = Http::new()
        .bind_handle(addr, server_handle, service)?
        .run_until(shutdown.map_err(|_| ()));
    Ok(run)
}

fn start_workload<K>(
    addr: Url,
    key_store: &K,
    handle: &Handle,
    shutdown: Receiver<()>,
) -> Result<Run, Error>
where
    K: 'static + KeyStore + Clone,
{
    let server_handle = handle.clone();
    let service = LoggingService::new(ApiVersionService::new(WorkloadService::new(
        key_store,
        Crypto::default(),
    )?));

    info!("Listening on {} with 1 thread for workload API.", addr);

    let run = Http::new()
        .bind_handle(addr, server_handle, service)?
        .run_until(shutdown.map_err(|_| ()));
    Ok(run)
}
