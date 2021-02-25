use anyhow::{anyhow, Result};
use obs;
use rpc::{
    obs_server::{Obs, ObsServer},
    SetStreamReply, SetStreamRequest, TestReply, TestRequest,
};
use std::{
    ffi::{c_void, CStr, CString},
    fs,
    ptr::{null, null_mut},
};
use tonic::{transport::Server, Request, Response, Status};

mod hl;

use hl::{Array, Data, Session, SessionSettings};

#[derive(Default)]
pub struct ThisServer {}

fn set_url(source: &str, url: &str) -> Result<()> {
    unsafe {
        let source = obs::obs_get_source_by_name(cstr!(source));
        let mut settings = Data::from_raw(obs::obs_source_get_settings(source));

        let mut playlist = Array::new()?;
        let mut item = Data::new()?;
        item.set_bool("hidden", false)?;
        item.set_bool("selected", false)?;
        item.set_string("value", url)?;
        playlist.push_back(item);

        settings.set_array("playlist", playlist)?;
        obs::obs_source_update(source, settings.as_mut_ptr());
        // let str = obs::obs_data_get_json(settings.as_mut_ptr());
        // let json = CStr::from_ptr(str);
        //println!("settings: {:?}", json);

        obs::obs_source_release(source);
    }
    Ok(())
}

#[tonic::async_trait]
impl Obs for ThisServer {
    async fn test(
        &self,
        request: Request<TestRequest>,
    ) -> std::result::Result<Response<TestReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let url = request.into_inner().text;
        //set_url(&url).map_err(|e| Status::new(tonic::Code::Unknown, format!("{}", e)))?;

        let reply = TestReply {
            text: format!("Hello {}!", &url),
        };
        Ok(Response::new(reply))
    }

    async fn set_stream(
        &self,
        request: Request<SetStreamRequest>,
    ) -> std::result::Result<Response<SetStreamReply>, Status> {
        println!("Got set stream request from {:?}", request.remote_addr());

        let req = request.into_inner();
        let source = req.source;
        let url = req.url;
        set_url(&source, &url).map_err(|e| Status::new(tonic::Code::Unknown, format!("{}", e)))?;

        let reply = SetStreamReply {};
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let settings = SessionSettings {
        base_width: 1280,
        base_height: 720,
        output_width: 1280,
        output_height: 720,
        fps: 30,
        audio_samples_per_sec: 48000,
        stream_url: "rtmp://localhost:1935/live/".into(),
        stream_key: "key".into(),
    };
    let mut session = Session::new(&settings)?;

    let json_str = fs::read_to_string("obs.json")?;
    session.load_config_json(&json_str)?;

    session.start()?;

    let addr = "[::1]:50051".parse().unwrap();

    println!("GreeterServer listening on {}", addr);

    loop {
        let server = ThisServer::default();
        Server::builder()
            .add_service(ObsServer::new(server))
            .serve(addr)
            .await?;
        println!("loop");
    }

    Ok(())
}
