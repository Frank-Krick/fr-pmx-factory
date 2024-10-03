use fr_logging::Logger;
use fr_pmx_config_lib::FactoryConfig;
use tonic::transport::Channel;

use super::{
    pmx::{
        channel_strip::PmxChannelStripType,
        mod_host::{mod_host_proxy_client::ModHostProxyClient, plugins::PmxPlugin},
        pipewire::pipewire_client::PipewireClient,
    },
    utils::{connect_plugins, create_plugin},
};

pub struct ChannelStripPlugins {
    pub cross_fader: Option<PmxPlugin>,
    pub saturator: PmxPlugin,
    pub compressor: PmxPlugin,
    pub equalizer: PmxPlugin,
    pub gain: PmxPlugin,
}

pub async fn create_channel_strip(
    channel_type: PmxChannelStripType,
    config: FactoryConfig,
    mod_host_client: ModHostProxyClient<Channel>,
    pipewire_client: PipewireClient<Channel>,
    logger: &Logger,
) -> ChannelStripPlugins {
    let plugins = create_channel_strip_plugins(channel_type, config, mod_host_client, logger).await;
    connect_channel_internals(channel_type, &plugins, pipewire_client, logger).await;
    plugins
}

async fn connect_channel_internals(
    channel_type: PmxChannelStripType,
    plugins: &ChannelStripPlugins,
    pipewire_client: PipewireClient<Channel>,
    logger: &Logger,
) {
    if channel_type == PmxChannelStripType::CrossFaded {
        connect_plugins(
            &plugins.cross_fader.clone().unwrap(),
            &plugins.saturator,
            pipewire_client.clone(),
            logger,
        )
        .await;
    }

    connect_plugins(
        &plugins.saturator,
        &plugins.compressor,
        pipewire_client.clone(),
        logger,
    )
    .await;

    connect_plugins(
        &plugins.compressor,
        &plugins.equalizer,
        pipewire_client.clone(),
        logger,
    )
    .await;

    connect_plugins(
        &plugins.equalizer,
        &plugins.gain,
        pipewire_client.clone(),
        logger,
    )
    .await;
}

async fn create_channel_strip_plugins(
    channel_type: PmxChannelStripType,
    config: FactoryConfig,
    client: ModHostProxyClient<Channel>,
    logger: &Logger,
) -> ChannelStripPlugins {
    let clones = (
        client.clone(),
        client.clone(),
        client.clone(),
        client.clone(),
        client.clone(),
    );
    ChannelStripPlugins {
        cross_fader: match channel_type {
            PmxChannelStripType::Basic => None,
            PmxChannelStripType::CrossFaded => Some(
                create_plugin(
                    config.channel_strip.cross_fader_plugin_url.clone(),
                    clones.0,
                    logger,
                )
                .await,
            ),
        },
        saturator: create_plugin(
            config.channel_strip.saturator_plugin_url.clone(),
            clones.1,
            logger,
        )
        .await,
        compressor: create_plugin(
            config.channel_strip.cross_fader_plugin_url.clone(),
            clones.2,
            logger,
        )
        .await,
        equalizer: create_plugin(
            config.channel_strip.equalizer_plugin_url.clone(),
            clones.3,
            logger,
        )
        .await,
        gain: create_plugin(
            config.channel_strip.gain_plugin_url.clone(),
            clones.4,
            logger,
        )
        .await,
    }
}
