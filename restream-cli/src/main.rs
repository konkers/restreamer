use anyhow::Result;
use rpc::{
    obs_client::ObsClient, GetSourceStatusRequest, SetSourceVolumeRequest, SetStreamRequest,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Opt {
    SetStream { source_name: String, url: String },
    SourceStatus {},
    Volume { source_name: String, volume: f32 },
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    match opt {
        Opt::SetStream { source_name, url } => {
            let mut client = ObsClient::connect("http://[::1]:50051").await?;

            let request = tonic::Request::new(SetStreamRequest {
                source: source_name,
                url,
            });

            let _response = client.set_stream(request).await?;
        }
        Opt::Volume {
            source_name,
            volume,
        } => {
            let mut client = ObsClient::connect("http://[::1]:50051").await?;

            let request = tonic::Request::new(SetSourceVolumeRequest {
                source: source_name,
                volume,
            });

            let _response = client.set_source_volume(request).await?;
        }
        Opt::SourceStatus {} => {
            let mut client = ObsClient::connect("http://[::1]:50051").await?;

            let request = tonic::Request::new(GetSourceStatusRequest {});

            let response = client.get_source_status(request).await?;

            println!("{:#?}", response);
        }
    }

    Ok(())
}
