use fr_logging::Logger;
use tonic::transport::Channel;
use tonic::Request;

use super::pmx::mod_host::CreatePluginInstanceRequest;
use super::pmx::mod_host::{
    mod_host_proxy_client::ModHostProxyClient,
    plugins::{PmxPlugin, PmxPluginType},
};
use super::pmx::pipewire::pipewire_client::PipewireClient;
use super::pmx::pipewire::CreateLinkByNameRequest;

pub async fn create_plugin(
    uri: String,
    mut client: ModHostProxyClient<Channel>,
    logger: &Logger,
) -> PmxPlugin {
    logger.log_info("Creating plugin");
    let request = CreatePluginInstanceRequest {
        plugin_type: PmxPluginType::Lv2 as i32,
        plugin_uri: uri,
    };
    let response = client.create_plugin_instance(request).await.unwrap();
    response.into_inner().plugin.unwrap()
}

pub async fn connect_cross_fader_left(
    output: &PmxPlugin,
    cross_fader: &PmxPlugin,
    pipewire_client: PipewireClient<Channel>,
    logger: &Logger,
) {
    connect_plugins(output, cross_fader, pipewire_client, logger).await;
}

pub async fn connect_cross_fader_right(
    output: &PmxPlugin,
    cross_fader: &PmxPlugin,
    mut pipewire_client: PipewireClient<Channel>,
    logger: &Logger,
) {
    logger.log_info("Connecting plugins");
    let request = Request::new(CreateLinkByNameRequest {
        output_port_id: 0,
        input_port_id: 2,
        output_node_name: output.name.clone(),
        input_node_name: cross_fader.name.clone(),
    });
    pipewire_client.create_link_by_name(request).await.unwrap();
    let request = Request::new(CreateLinkByNameRequest {
        output_port_id: 1,
        input_port_id: 3,
        output_node_name: output.name.clone(),
        input_node_name: cross_fader.name.clone(),
    });
    pipewire_client.create_link_by_name(request).await.unwrap();
}

pub async fn connect_plugins(
    output: &PmxPlugin,
    input: &PmxPlugin,
    mut pipewire_client: PipewireClient<Channel>,
    logger: &Logger,
) {
    logger.log_info("Connecting plugins");
    let request = Request::new(CreateLinkByNameRequest {
        output_port_id: 0,
        input_port_id: 0,
        output_node_name: output.name.clone(),
        input_node_name: input.name.clone(),
    });
    pipewire_client.create_link_by_name(request).await.unwrap();
    let request = Request::new(CreateLinkByNameRequest {
        output_port_id: 1,
        input_port_id: 1,
        output_node_name: output.name.clone(),
        input_node_name: input.name.clone(),
    });
    pipewire_client.create_link_by_name(request).await.unwrap();
}
