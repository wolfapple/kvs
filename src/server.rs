use crate::engine::KvsEngine;
use crate::protocol::{Request, Response};
use crate::Result;
use log::{debug, error};
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use crate::thread_pool::ThreadPool;

pub struct KvsServer<E: KvsEngine, P: ThreadPool> {
    engine: E,
    pool: P,
}

impl<E: KvsEngine, P: ThreadPool> KvsServer<E, P> {
    pub fn new(engine: E, pool: P) -> Self {
        KvsServer { engine, pool }
    }

    pub fn run<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let engine = self.engine.clone();
                    self.pool.spawn(move || {
                        if let Err(e) = handle_client(engine, stream) {
                            error!("Error handling client: {}", e);
                        }
                    })
                }
                Err(e) => error!("Connection failed: {}", e),
            }
        }
        Ok(())
    }
}

fn handle_client<E: KvsEngine>(engine: E, stream: TcpStream) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let req_stream = serde_json::Deserializer::from_reader(reader).into_iter::<Request>();

    for req in req_stream {
        let req = req?;
        debug!("Receive request from {}: {:?}", stream.peer_addr()?, req);
        let resp = match req {
            Request::Get { key } => match engine.get(key) {
                Ok(value) => Response::Ok(value),
                Err(e) => Response::Err(e.to_string()),
            },
            Request::Set { key, value } => match engine.set(key, value) {
                Ok(_) => Response::Ok(None),
                Err(e) => Response::Err(e.to_string()),
            },
            Request::Remove { key } => match engine.remove(key) {
                Ok(_) => Response::Ok(None),
                Err(e) => Response::Err(e.to_string()),
            },
        };
        serde_json::to_writer(&mut writer, &resp)?;
        writer.flush()?;
        debug!("Response sent to {}: {:?}", stream.peer_addr()?, resp);
    }
    Ok(())
}