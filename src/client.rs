use crate::protocol::{Request, Response};
use crate::{KvsError, Result};
use serde::Deserialize;
use serde_json::de::{Deserializer, IoRead};
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpStream, ToSocketAddrs};

pub struct KvsClient {
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>,
}

impl KvsClient {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let reader = TcpStream::connect(addr)?;
        let writer = reader.try_clone()?;
        Ok(KvsClient {
            reader: Deserializer::from_reader(BufReader::new(reader)),
            writer: BufWriter::new(writer),
        })
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let req = Request::Get { key };
        serde_json::to_writer(&mut self.writer, &req)?;
        self.writer.flush()?;
        let resp = Response::deserialize(&mut self.reader)?;
        match resp {
            Response::Ok(value) => Ok(value),
            Response::Err(msg) => Err(KvsError::StringError(msg)),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let req = Request::Set { key, value };
        serde_json::to_writer(&mut self.writer, &req)?;
        self.writer.flush()?;
        let resp = Response::deserialize(&mut self.reader)?;
        match resp {
            Response::Ok(_) => Ok(()),
            Response::Err(msg) => Err(KvsError::StringError(msg)),
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        let req = Request::Remove { key };
        serde_json::to_writer(&mut self.writer, &req)?;
        self.writer.flush()?;
        let resp = Response::deserialize(&mut self.reader)?;
        match resp {
            Response::Ok(_) => Ok(()),
            Response::Err(msg) => Err(KvsError::StringError(msg)),
        }
    }
}
