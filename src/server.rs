use crate::engine::KvsEngine;
use crate::protocol::{Request, Response};
use crate::Result;
use log::error;
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

pub struct KvsServer<E: KvsEngine> {
    engine: E,
}

impl<E: KvsEngine> KvsServer<E> {
    pub fn new(engine: E) -> Self {
        KvsServer { engine }
    }

    pub fn run<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.handle_client(stream) {
                        error!("Error handling client: {}", e);
                    }
                }
                Err(e) => error!("Connection failed: {}", e),
            }
        }
        Ok(())
    }

    fn handle_client(&mut self, stream: TcpStream) -> Result<()> {
        let reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);
        let req_stream = serde_json::Deserializer::from_reader(reader).into_iter::<Request>();

        for req in req_stream {
            let req = req?;
            let resp = match req {
                Request::Get { key } => match self.engine.get(key) {
                    Ok(value) => Response::Ok(value),
                    Err(e) => Response::Err(e.to_string()),
                },
                Request::Set { key, value } => match self.engine.set(key, value) {
                    Ok(_) => Response::Ok(None),
                    Err(e) => Response::Err(e.to_string()),
                },
                Request::Remove { key } => match self.engine.remove(key) {
                    Ok(_) => Response::Ok(None),
                    Err(e) => Response::Err(e.to_string()),
                },
            };
            serde_json::to_writer(&mut writer, &resp)?;
            writer.flush()?;
        }
        Ok(())
    }
}
