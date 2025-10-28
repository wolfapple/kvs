use crate::protocol::{Request, Response};
use crate::Result;
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpStream, ToSocketAddrs};

pub struct KvsClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl KvsClient {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(KvsClient {
            reader: BufReader::new(stream.try_clone()?),
            writer: BufWriter::new(stream),
        })
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let req = Request::Get { key };
        serde_json::to_writer(&mut self.writer, &req)?;
        self.writer.flush()?;
        let resp = serde_json::from_reader(&mut self.reader)?;
        match resp {
            Response::Ok(value) => Ok(value),
            Response::Err(msg) => Err(crate::KvsError::StringError(msg)),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let req = Request::Set { key, value };
        serde_json::to_writer(&mut self.writer, &req)?;
        self.writer.flush()?;
        let resp = serde_json::from_reader(&mut self.reader)?;
        match resp {
            Response::Ok(_) => Ok(()),
            Response::Err(msg) => Err(crate::KvsError::StringError(msg)),
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        let req = Request::Remove { key };
        serde_json::to_writer(&mut self.writer, &req)?;
        self.writer.flush()?;
        let resp = serde_json::from_reader(&mut self.reader)?;
        match resp {
            Response::Ok(_) => Ok(()),
            Response::Err(msg) => Err(crate::KvsError::StringError(msg)),
        }
    }
}
