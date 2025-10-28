use clap::{Parser, Subcommand};
use kvs::{KvsClient, Result};
use std::net::SocketAddr;

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
    #[arg(
        short,
        long,
        global = true,
        name = "IP:PORT",
        help = "Sets the server address",
        default_value = "127.0.0.1:4000",
    )]
    addr: SocketAddr,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Set the value of a string key to a string", name = "set")]
    Set {
        #[arg(name = "KEY", help = "A string key")]
        key: String,
        #[arg(name = "VALUE", help = "The string value of the key")]
        value: String,
    },
    #[command(about = "Get the string value of a given string key", name = "get")]
    Get {
        #[arg(name = "KEY", help = "A string key")]
        key: String
    },
    #[command(about = "Remove a given string key", name = "rm")]
    Remove {
        #[arg(name = "KEY", help = "A string key")]
        key: String
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut client = KvsClient::connect(args.addr)?;
    match args.cmd {
        Commands::Set { key, value } => {
            client.set(key, value)?;
        }
        Commands::Get { key } => {
            if let Some(value) = client.get(key)? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        Commands::Remove { key } => {
            client.remove(key)?;
        }
    }
    Ok(())
}