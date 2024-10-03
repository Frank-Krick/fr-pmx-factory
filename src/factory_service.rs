use std::result::Result;

use fr_logging::Logger;
use pmx::factory::channel_strip::PmxChannelStrip;
use pmx::factory::output_stage::PmxOutputStage;
use pmx::factory::pmx_factory_server::{PmxFactory, PmxFactoryServer};
use pmx::factory::{CreateChannelStripRequest, CreateOutputStageRequest};

use tonic::{Request, Response, Status};

use crate::factory::pmx::channel_strip::PmxChannelStripType;
use crate::factory::FactoryRequest;

pub mod pmx {
    pub mod factory {
        tonic::include_proto!("pmx.factory");

        pub mod channel_strip {
            tonic::include_proto!("pmx.factory.channel_strip");
        }

        pub mod output_stage {
            tonic::include_proto!("pmx.factory.output_stage");
        }
    }
}

pub struct FactoryService {
    sender: tokio::sync::mpsc::UnboundedSender<FactoryRequest>,
    logger: Logger,
}

impl FactoryService {
    pub fn new(sender: tokio::sync::mpsc::UnboundedSender<FactoryRequest>, logger: Logger) -> Self {
        FactoryService { sender, logger }
    }

    pub fn new_server(
        sender: tokio::sync::mpsc::UnboundedSender<FactoryRequest>,
        logger: Logger,
    ) -> PmxFactoryServer<FactoryService> {
        PmxFactoryServer::new(FactoryService::new(sender, logger))
    }
}

#[tonic::async_trait]
impl PmxFactory for FactoryService {
    async fn create_channel_strip(
        &self,
        request: Request<CreateChannelStripRequest>,
    ) -> Result<Response<PmxChannelStrip>, Status> {
        self.logger
            .log_info("Received create channel strip request");
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
        let inner = request.into_inner();
        let factory_request = FactoryRequest::CreateChannelStrip {
            response_sender,
            name: inner.name,
            channel_type: PmxChannelStripType::try_from(inner.channel_type).unwrap(),
        };
        self.sender.send(factory_request).unwrap();
        let factory_response = response_receiver.await.unwrap();
        Ok(Response::new(PmxChannelStrip {
            id: factory_response.id,
            saturator_plugin_id: factory_response.saturator.id,
            compressor_plugin_id: factory_response.compressor.id,
            equalizer_plugin_id: factory_response.equalizer.id,
            name: factory_response.name,
            channel_type: factory_response.channel_type as i32,
            cross_fader_plugin_id: match factory_response.cross_fader {
                Some(fader) => Some(fader.id),
                None => None,
            },
            gain_plugin_id: factory_response.gain.id,
        }))
    }

    async fn create_output_stage(
        &self,
        request: Request<CreateOutputStageRequest>,
    ) -> Result<Response<PmxOutputStage>, Status> {
        self.logger.log_info("Received create output stage request");
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
        let inner = request.into_inner();
        let factory_request = FactoryRequest::CreateOutputStage {
            name: inner.name,
            response_sender,
        };
        self.sender.send(factory_request).unwrap();
        let factory_response = response_receiver.await.unwrap();
        Ok(Response::new(PmxOutputStage {
            id: factory_response.id,
            name: factory_response.name,
            left_channel_strip_id: factory_response.left_channel_strip_id,
            right_channel_strip_id: factory_response.right_channel_strip_id,
            cross_fader_plugin_id: factory_response.cross_fader_plugin_id,
        }))
    }
}
