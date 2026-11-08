#![allow(unused)]
use std::{
    env,
    error::{self, Error},
    fs::OpenOptions,
    io::{Write, stdin, stdout},
    sync::{Arc, RwLock},
    thread,
    time::Instant,
};

use engine::{
    engine::Engine, engine_error::EngineError, memtable::Value, skiplist::SkipListKV,
    sstable::iterator::SstableIterator, tests::test_write_read_compact,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{
    Request, Response, Status, Streaming,
    server::{ServerStreamingService, StreamingService},
    transport::Server,
};
use tracing_subscriber::EnvFilter;

use crate::sthirayama::{
    DelRequest, DelResponse, GetRequest, GetResponse, RestoreRequest, RestoreResponse, SetRequest,
    SetResponse, SnapshotRequest, SnapshotResponse,
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
            e.set(&key, &val);
            response = true;
        }
        let reply = SetResponse { ok: response };
        Ok(Response::new(reply))
    }
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let GetRequest { key } = request.into_inner();
        let mut value = "Error".as_bytes().to_vec();
        if let Ok(e) = self.engine.read() {
            if let Ok(x) = e.get(&key) {
                if let Some(val) = x {
                    value = val.clone();
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
            e.del(&key);
            response = true;
        }
        let reply = DelResponse { ok: response };
        Ok(Response::new(reply))
    }

    type SnapshotStream = ReceiverStream<Result<SnapshotResponse, Status>>;
    async fn snapshot(
        &self,
        request: Request<SnapshotRequest>,
    ) -> Result<Response<Self::SnapshotStream>, Status> {
        let (tx, rx) = mpsc::channel(16);
        let engine = self.engine.clone();
        tokio::spawn(async move {
            let mut cursor: Option<SkipListKV<Vec<u8>, Vec<u8>>> = None;
            loop {
                let iter_chunk: Vec<_> = {
                    let iter_guard = engine.read().expect("Engine Rw poisoned");

                    let iter = iter_guard
                        .iter()
                        .map_err(|e| Status::internal(format!("iter error {:?}", e)))
                        .unwrap();
                    iter.skip_while(|kv| {
                        if let Some(cu) = &cursor {
                            return kv.key <= cu.key;
                        } else {
                            return false;
                        }
                    })
                    .take(1000)
                    .collect()
                };
                if iter_chunk.is_empty() {
                    break;
                }
                let last_index = iter_chunk.len() - 1;
                cursor = Some(iter_chunk[last_index].clone());
                for SkipListKV {
                    key,
                    value,
                    sequence,
                } in &iter_chunk
                {
                    let raw_value = match Value::from_bytes(value) {
                        Ok(Value::Data(val)) => val,
                        _ => value.clone(),
                    };
                    if let Err(_) = tx
                        .send(Ok(SnapshotResponse {
                            key: key.clone(),
                            value: raw_value,
                            sequence: u64::from_le_bytes(sequence.clone()),
                        }))
                        .await
                    {
                        tracing::info!("client disconnected");
                        break;
                    }
                }
                if iter_chunk.len() < 1000 {
                    break;
                }
            }
        });

        // let iter_guard = self.engine.read().expect("Engine Rw poisoned");
        // let iter = iter_guard
        //     .iter()
        //     .map_err(|e| Status::internal(format!("iter error {:?}", e)))?;
        // for SkipListKV {
        //     key,
        //     value,
        //     sequence,
        // } in iter
        // {
        //     if let Err(_) = tx
        //         .send(Ok(SnapshotResponse {
        //             key,
        //             value,
        //             sequence: sequence.to_vec(),
        //         }))
        //         .await
        //     {
        //         tracing::info!("client disconnected");
        //         break;
        //     }
        // }
        Ok(Response::new(ReceiverStream::new(rx)))
    }
    async fn restore(
        &self,
        request: Request<Streaming<RestoreRequest>>,
    ) -> Result<Response<RestoreResponse>, Status> {
        let mut stream = request.into_inner();
        while let Some(chunk) = stream.message().await? {
            let RestoreRequest {
                key,
                value,
                sequence,
            } = chunk;
            if let Ok(mut engine) = self.engine.write() {
                engine.set_with_sequence(&key, &value, sequence as usize);
            }
        }
        Ok(Response::new(RestoreResponse {
            message: "restored".to_string(),
        }))
    }
}
async fn run_grpc() -> Result<(), EngineError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        tracing::error!("No path or port provided. Usage: <path> <port>");
        return Ok(());
    }
    let path = &args[1];
    let port = &args[2];

    let engine = Engine::new(path)?;
    let shared_engine = Arc::new(RwLock::new(engine));
    let addr = format!("127.0.0.1:{}", port).parse()?;
    let sthira = MySthirayama::new(shared_engine);
    tracing::info!("Sthirayama listening on {}", addr);
    Server::builder()
        .add_service(SthirayamaServer::new(sthira))
        .serve(addr)
        .await?;
    Ok(())
}
fn run_engine_iter() -> Result<(), EngineError> {
    let engine = Engine::new("./main")?;
    for i in engine.iter()? {
        println!("{}", i);
    }
    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), EngineError> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    // test_write_read_compact()?;
    // run_engine_iter()?;
    run_grpc().await?;
    Ok(())
}
