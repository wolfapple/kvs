use std::process::exit;
use clap::{Parser, Subcommand};

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

fn main() {
    let args = Args::parse();
    match args.cmd {
        Commands::Set { key, value } => {
            eprintln!("unimplemented");
            exit(1);
        }
        Commands::Get { key } => {
            eprintln!("unimplemented");
            exit(1);
        }
        Commands::Rm { key } => {
            eprintln!("unimplemented");
            exit(1);
        }
    }
}