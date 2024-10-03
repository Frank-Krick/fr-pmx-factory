use channel_strip_factory::{create_channel_strip, ChannelStripPlugins};
use fr_logging::Logger;
use pmx::{
    channel_strip::PmxChannelStripType, mod_host::plugins::PmxPlugin,
    pmx_registry_client::PmxRegistryClient, RegisterOutputStageRequest,
};
use tonic::{transport::Channel, Request};

mod channel_strip_factory;
mod utils;

pub mod pmx {
    tonic::include_proto!("pmx");

    pub mod mod_host {
        tonic::include_proto!("pmx.mod_host");

        pub mod plugins {
            tonic::include_proto!("pmx.mod_host.plugins");
        }
    }

    pub mod input {
        tonic::include_proto!("pmx.input");
    }

    pub mod output {
        tonic::include_proto!("pmx.output");
    }

    pub mod plugin {
        tonic::include_proto!("pmx.plugin");
    }

    pub mod channel_strip {
        tonic::include_proto!("pmx.channel_strip");
    }

    pub mod output_stage {
        tonic::include_proto!("pmx.output_stage");
    }

    pub mod looper {
        tonic::include_proto!("pmx.looper");
    }

    pub mod pipewire {
        tonic::include_proto!("pmx.pipewire");

        pub mod node {
            tonic::include_proto!("pmx.pipewire.node");
        }

        pub mod port {
            tonic::include_proto!("pmx.pipewire.port");
        }

        pub mod application {
            tonic::include_proto!("pmx.pipewire.application");
        }

        pub mod device {
            tonic::include_proto!("pmx.pipewire.device");
        }

        pub mod link {
            tonic::include_proto!("pmx.pipewire.link");
        }
    }
}

#[derive(Debug)]
pub struct CreateChannelStripResponse {
    pub id: u32,
    pub name: String,
    pub cross_fader: Option<PmxPlugin>,
    pub saturator: PmxPlugin,
    pub compressor: PmxPlugin,
    pub equalizer: PmxPlugin,
    pub gain: PmxPlugin,
    pub channel_type: PmxChannelStripType,
}

#[derive(Debug)]
pub struct CreateOutputStageResponse {
    pub id: u32,
    pub name: String,
    pub left_channel_strip_id: u32,
    pub right_channel_strip_id: u32,
    pub cross_fader_plugin_id: u32,
}

pub enum FactoryRequest {
    CreateChannelStrip {
        name: String,
        response_sender: tokio::sync::oneshot::Sender<CreateChannelStripResponse>,
        channel_type: PmxChannelStripType,
    },
    CreateOutputStage {
        name: String,
        response_sender: tokio::sync::oneshot::Sender<CreateOutputStageResponse>,
    },
}

pub struct Factory {
    receiver: tokio::sync::mpsc::UnboundedReceiver<FactoryRequest>,
    mod_host_client:
        pmx::mod_host::mod_host_proxy_client::ModHostProxyClient<tonic::transport::Channel>,
    registry_client: pmx::pmx_registry_client::PmxRegistryClient<tonic::transport::Channel>,
    pipewire_client: pmx::pipewire::pipewire_client::PipewireClient<tonic::transport::Channel>,
    config: fr_pmx_config_lib::FactoryConfig,
    next_channel_strip_id: u32,
    logger: Logger,
}

impl Factory {
    pub fn new(
        receiver: tokio::sync::mpsc::UnboundedReceiver<FactoryRequest>,
        mod_host_client: pmx::mod_host::mod_host_proxy_client::ModHostProxyClient<
            tonic::transport::Channel,
        >,
        registry_client: pmx::pmx_registry_client::PmxRegistryClient<tonic::transport::Channel>,
        pipewire_client: pmx::pipewire::pipewire_client::PipewireClient<tonic::transport::Channel>,
        config: fr_pmx_config_lib::FactoryConfig,
        logger: Logger,
    ) -> Self {
        Factory {
            receiver,
            mod_host_client,
            registry_client,
            pipewire_client,
            config,
            next_channel_strip_id: 0,
            logger,
        }
    }

    pub async fn run(&mut self) {
        self.logger.log_info("Starting factory run loop");
        loop {
            let request = self.receiver.recv().await.unwrap();
            self.logger.log_info("Processing request");
            match request {
                FactoryRequest::CreateChannelStrip {
                    name,
                    response_sender,
                    channel_type,
                } => {
                    let (id, plugins) = self
                        .create_and_register_channel_strip(name.clone(), channel_type)
                        .await;
                    response_sender
                        .send(CreateChannelStripResponse {
                            id,
                            name: name.clone(),
                            cross_fader: plugins.cross_fader,
                            saturator: plugins.saturator,
                            compressor: plugins.compressor,
                            equalizer: plugins.equalizer,
                            gain: plugins.gain,
                            channel_type,
                        })
                        .unwrap();
                }
                FactoryRequest::CreateOutputStage {
                    name,
                    response_sender,
                } => {
                    let (left_id, left_plugins) = self
                        .create_and_register_channel_strip(
                            String::from("Left Stage"),
                            PmxChannelStripType::Basic,
                        )
                        .await;
                    let (right_id, right_plugins) = self
                        .create_and_register_channel_strip(
                            String::from("Right Stage"),
                            PmxChannelStripType::Basic,
                        )
                        .await;
                    let cross_fader_plugin = utils::create_plugin(
                        self.config.channel_strip.cross_fader_plugin_url.clone(),
                        self.mod_host_client.clone(),
                        &self.logger,
                    )
                    .await;
                    utils::connect_cross_fader_left(
                        &left_plugins.gain,
                        &cross_fader_plugin,
                        self.pipewire_client.clone(),
                        &self.logger,
                    )
                    .await;
                    utils::connect_cross_fader_right(
                        &right_plugins.gain,
                        &cross_fader_plugin,
                        self.pipewire_client.clone(),
                        &self.logger,
                    )
                    .await;

                    let registry_request = RegisterOutputStageRequest {
                        name: name.clone(),
                        left_channel_strip_id: left_id,
                        right_channel_strip_id: right_id,
                        cross_fader_plugin_id: cross_fader_plugin.id,
                    };

                    let registry_response = self
                        .registry_client
                        .register_output_stage(Request::new(registry_request))
                        .await
                        .unwrap();

                    let registration = registry_response.into_inner();

                    response_sender
                        .send(CreateOutputStageResponse {
                            id: registration.id,
                            name: name.clone(),
                            left_channel_strip_id: left_id,
                            right_channel_strip_id: right_id,
                            cross_fader_plugin_id: cross_fader_plugin.id,
                        })
                        .unwrap();
                }
            }
        }
    }

    async fn create_and_register_channel_strip(
        &mut self,
        name: String,
        channel_type: PmxChannelStripType,
    ) -> (u32, ChannelStripPlugins) {
        let id = self.next_channel_strip_id;
        self.next_channel_strip_id += 1;
        let plugins = create_channel_strip(
            channel_type,
            self.config.clone(),
            self.mod_host_client.clone(),
            self.pipewire_client.clone(),
            &self.logger,
        )
        .await;
        self.register_channel_strip(
            id,
            name.clone(),
            channel_type,
            &plugins,
            self.registry_client.clone(),
        )
        .await;
        (id, plugins)
    }

    async fn register_channel_strip(
        &self,
        id: u32,
        name: String,
        channel_type: PmxChannelStripType,
        plugins: &ChannelStripPlugins,
        client: PmxRegistryClient<Channel>,
    ) {
        let registry_channel_strip = pmx::channel_strip::PmxChannelStrip {
            id,
            name,
            channel_strip_type: channel_type as i32,
            cross_fader_plugin_id: plugins.cross_fader.clone().map(|c| c.id),
            saturator_plugin_id: plugins.saturator.id,
            compressor_plugin_id: plugins.compressor.id,
            equalizer_plugin_id: plugins.equalizer.id,
            gain_plugin_id: plugins.gain.id,
        };
        let mut client = client;
        client
            .register_channel_strip(Request::new(pmx::RegisterChannelStripRequest {
                channel_strip: Some(registry_channel_strip.clone()),
            }))
            .await
            .unwrap();
    }
}
