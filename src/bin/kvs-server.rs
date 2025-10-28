use clap::Parser;
use env_logger::Env;
use kvs::{Engine, Result};
use log::info;
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
        default_value_t = Engine::Kvs,
        name = "ENGINE-NAME",
        help = "Sets the storage engine"
    )]
    engine: Engine,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {}", args.engine);
    info!("Listening on {}", args.addr);

    match args.engine {
        Engine::Kvs => {}
        Engine::Sled => {}
    }
    Ok(())
}
