use clap::{Parser, Subcommand};
use pmx::factory::{
    channel_strip::PmxChannelStripType, pmx_factory_client::PmxFactoryClient,
    CreateChannelStripRequest,
};
use tonic::Request;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Commands>,
}

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

#[derive(Subcommand)]
enum Commands {
    CreateChannelStrip {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        basic: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service_urls = fr_pmx_config_lib::read_service_urls();
    let cli_arguments = Arguments::parse();

    if let Some(command) = cli_arguments.command {
        match command {
            Commands::CreateChannelStrip { name, basic } => {
                let mut client = PmxFactoryClient::connect(service_urls.pmx_factory_url).await?;
                let request = Request::new(CreateChannelStripRequest {
                    name,
                    channel_type: match basic {
                        true => PmxChannelStripType::Basic as i32,
                        false => PmxChannelStripType::CrossFaded as i32,
                    },
                });
                let response = client.create_channel_strip(request).await?;
                println!("{response:#?}");
            }
        }
    }

    Ok(())
}
