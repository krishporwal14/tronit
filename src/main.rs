use anyhow::Result;
use clap::{Parser, Subcommand};

mod repo;
mod utils;
mod object;

mod commands {
    pub mod init;
    pub mod hash_object;
    pub mod cat_file;
    pub mod add;
    pub mod commit;
    pub mod log;
}

use commands::*;

#[derive(Parser)]
#[command(name="tronit")]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    Init,
    HashObject { file: String },
    CatFile { hash: String },
    Add { path: String },
    Commit { 
        #[arg(short, long)]
        message: String,
        #[arg(long)]
        author_name: Option<String>,
        #[arg(long)]
        author_email: Option<String>,
    },
    Log
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init::run()?,
        Commands::HashObject { file } => hash_object::run(&file)?,
        Commands::CatFile { hash } => cat_file::run(&hash)?,
        Commands::Add { path } => add::run(&path)?,
        Commands::Commit { message, author_name, author_email } => commit::run(&message, author_name.as_deref(), author_email.as_deref())?,
        Commands::Log => log::run()?,
    }

    Ok(())
}
