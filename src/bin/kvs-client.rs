use clap::{Parser, Subcommand};
use kvs::{KvStore, KvsError, Result};
use std::env::current_dir;
use std::net::SocketAddr;
use std::process::exit;

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
    match args.cmd {
        Commands::Set { key, value } => {
            let mut store = KvStore::open(current_dir()?)?;
            store.set(key, value)?;
        }
        Commands::Get { key } => {
            let mut store = KvStore::open(current_dir()?)?;
            if let Some(value) = store.get(key)? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        Commands::Remove { key } => {
            let mut store = KvStore::open(current_dir()?)?;
            match store.remove(key) {
                Ok(_) => {}
                Err(KvsError::KeyNotFound) => {
                    println!("Key not found");
                    exit(1);
                }
                Err(e) => return Err(e)
            }
        }
    }
    Ok(())
}