#![allow(unused)]
use std::{
    error::{self, Error},
    fs::OpenOptions,
    io::{Write, stdin, stdout},
    time::Instant,
};

use engine::{engine::Engine, engine_error::EngineError, sstable::iterator::SstableIterator};
use tonic::{Request, Response, Status, transport::Server};

use crate::sthirayama::{
    DelRequest, DelResponse, GetRequest, GetResponse, SetRequest, SetResponse,
    sthirayama_server::{Sthirayama, SthirayamaServer},
};
// use sthirayama::

pub mod sthirayama {
    tonic::include_proto!("sthirayama");
}
#[derive(Default, Debug)]
pub struct MySthirayama {}
#[tonic::async_trait]
impl Sthirayama for MySthirayama {
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let key = request.into_inner().key;
        let reply = SetResponse { ok: true };
        Ok(Response::new(reply))
    }
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let key = request.into_inner().key;
        let reply = GetResponse {
            val: "val".to_string(),
        };
        Ok(Response::new(reply))
    }
    async fn del(&self, request: Request<DelRequest>) -> Result<Response<DelResponse>, Status> {
        let key = request.into_inner().key;
        let reply = DelResponse { ok: true };
        Ok(Response::new(reply))
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let sthira = MySthirayama::default();
    println!("Sthirayama listening on {}", addr);
    Server::builder()
        .add_service(SthirayamaServer::new(sthira))
        .serve(addr)
        .await?;
    Ok(())
}
