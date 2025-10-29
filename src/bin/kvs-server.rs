use clap::Parser;
use env_logger::Env;
use kvs::{Engine, KvStore, KvsError, KvsServer, Result, SledKvsEngine};
use log::info;
use std::env::current_dir;
use std::fs::File;
use std::net::SocketAddr;

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[arg(
        short,
        long,
        name = "IP:PORT",
        help = "Sets the server address",
        default_value = "127.0.0.1:4000"
    )]
    addr: SocketAddr,
    #[arg(
        short,
        long,
        value_enum,
        name = "ENGINE-NAME",
        help = "Sets the storage engine"
    )]
    engine: Option<Engine>,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    let engine = get_engine(args.engine)?;

    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {}", engine);
    info!("Listening on {}", args.addr);

    match engine {
        Engine::Kvs => {
            let mut server = KvsServer::new(KvStore::open(current_dir()?)?);
            server.run(args.addr)?;
        }
        Engine::Sled => {
            let mut server = KvsServer::new(SledKvsEngine::open(current_dir()?)?);
            server.run(args.addr)?;
        }
    }
    Ok(())
}

fn get_engine(engine: Option<Engine>) -> Result<Engine> {
    let engine_path = current_dir()?.join(".engine");
    match engine {
        Some(engine) => {
            if engine_path.exists() {
                let last_engine: Engine = serde_json::from_reader(File::open(&engine_path)?)?;
                if engine != last_engine {
                    return Err(KvsError::EngineMismatch);
                }
            }
            serde_json::to_writer(File::create(&engine_path)?, &engine)?;
            Ok(engine)
        }
        None => {
            if engine_path.exists() {
                let last_engine: Engine = serde_json::from_reader(File::open(&engine_path)?)?;
                return Ok(last_engine);
            }
            let default_engine = Engine::Kvs;
            serde_json::to_writer(File::create(&engine_path)?, &default_engine)?;
            Ok(default_engine)
        }
    }
}