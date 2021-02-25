use anyhow::Result;
use rpc::{obs_client::ObsClient, SetStreamRequest, TestRequest};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Opt {
    SetStream { source_name: String, url: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    match opt {
        Opt::SetStream { source_name, url } => {
            let mut client = ObsClient::connect("http://[::1]:50051").await?;

            let request = tonic::Request::new(SetStreamRequest {
                source: source_name,
                url: url,
            });

            let response = client.set_stream(request).await?;
        }
    }

    Ok(())
}
