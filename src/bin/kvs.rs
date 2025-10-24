use clap::{Parser, Subcommand};
use kvs::{KvStore, KvsError, Result};
use std::env::current_dir;
use std::process::exit;

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
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
        Commands::Rm { key } => {
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
