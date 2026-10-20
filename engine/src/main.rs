#![allow(unused)]
use std::{
    env,
    error::{self, Error},
    fs::OpenOptions,
    io::{Write, stdin, stdout},
    sync::{Arc, RwLock},
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
#[derive(Debug)]
pub struct MySthirayama {
    engine: Arc<RwLock<Engine>>,
}
impl MySthirayama {
    pub fn new(engine: Arc<RwLock<Engine>>) -> Self {
        Self { engine }
    }
}
#[tonic::async_trait]
impl Sthirayama for MySthirayama {
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let mut response = false;
        let SetRequest { key, val } = request.into_inner();
        if let Ok(mut e) = self.engine.write() {
            e.set(&key.into_bytes(), &val.into_bytes());
            response = true;
        }
        let reply = SetResponse { ok: response };
        Ok(Response::new(reply))
    }
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let GetRequest { key } = request.into_inner();
        let mut value = "Error".to_string();
        if let Ok(e) = self.engine.read() {
            if let Ok(x) = e.get(&key.into_bytes()) {
                if let Some(val) = x {
                    value = str::from_utf8(&val).unwrap().to_string();
                }
            }
        }
        let reply = GetResponse { val: value };
        Ok(Response::new(reply))
    }
    async fn del(&self, request: Request<DelRequest>) -> Result<Response<DelResponse>, Status> {
        let key = request.into_inner().key;
        let mut response = false;
        if let Ok(mut e) = self.engine.write() {
            e.del(&key.into_bytes());
            response = true;
        }
        let reply = DelResponse { ok: response };
        Ok(Response::new(reply))
    }
}
#[tokio::main]
async fn main() -> Result<(), EngineError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No path provided");
        return Ok(());
    }
    let path = &args[1];

    let engine = Engine::new(path)?;
    let shared_engine = Arc::new(RwLock::new(engine));
    let addr = "127.0.0.1:50051".parse()?;
    let sthira = MySthirayama::new(shared_engine);
    println!("Sthirayama listening on {}", addr);
    Server::builder()
        .add_service(SthirayamaServer::new(sthira))
        .serve(addr)
        .await?;
    Ok(())
}
