use factory::{
    pmx::{
        mod_host::mod_host_proxy_client::ModHostProxyClient,
        pipewire::pipewire_client::PipewireClient, pmx_registry_client::PmxRegistryClient,
    },
    Factory,
};
use factory_service::{pmx::factory::pmx_factory_server::PmxFactoryServer, FactoryService};
use tonic::transport::Server;

mod factory;
mod factory_service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fr_logging::setup_logging();
    let (logging_sender, logging_receiver) = tokio::sync::mpsc::unbounded_channel();
    let logger_factory = fr_logging::LoggerFactory::new(logging_sender);

    let service_urls = fr_pmx_config_lib::read_service_urls();
    let (factory_request_sender, factory_request_receiver) = tokio::sync::mpsc::unbounded_channel();
    let factory_service_logger = logger_factory.new_logger(String::from("factory_service"));
    let service = PmxFactoryServer::new(FactoryService::new(
        factory_request_sender,
        factory_service_logger,
    ));
    let server = Server::builder().add_service(service).serve(
        service_urls
            .pmx_factory_url
            .replace("http://", "")
            .parse()
            .unwrap(),
    );

    let factory_config = fr_pmx_config_lib::read_factory_config();
    let mod_host_client = ModHostProxyClient::connect(service_urls.pmx_mod_host_proxy_url)
        .await
        .unwrap();
    let registry_client = PmxRegistryClient::connect(service_urls.pmx_registry_url)
        .await
        .unwrap();
    let pipewire_client = PipewireClient::connect(service_urls.pipewire_registry_url)
        .await
        .unwrap();
    let factory_logger = logger_factory.new_logger(String::from("factory"));
    let mut factory = Factory::new(
        factory_request_receiver,
        mod_host_client,
        registry_client,
        pipewire_client,
        factory_config,
        factory_logger,
    );

    tokio::select! {
        _ = server => {Ok(())}
        _ = factory.run() => {Ok(())}
        _ = fr_logging::run_logging_task(logging_receiver) => {Ok(())}
    }
}
