use anyhow::Result;
use rpc::{obs_client::ObsClient, SetSourceVolumeRequest, SetStreamRequest, TestRequest};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Opt {
    SetStream { source_name: String, url: String },
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

            let response = client.set_stream(request).await?;
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

            let response = client.set_source_volume(request).await?;
        }
    }

    Ok(())
}
