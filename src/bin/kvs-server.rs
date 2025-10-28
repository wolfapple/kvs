use clap::Parser;
use kvs::Result;
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
        name = "ENGINE-NAME",
        help = "kvs | sled",
        default_value = "kvs"
    )]
    engine: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("Starting server on {}", args.addr);
    Ok(())
}
