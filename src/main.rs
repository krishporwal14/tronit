use anyhow::Result;
use clap::{Parser, Subcommand};
use tronit::commands;

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
    Log,
    Status,
    Branch {
        name: Option<String>,
    },
    Switch {
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

     match cli.command {
        Commands::Init => commands::init::run()?,
        Commands::HashObject { file } => commands::hash_object::run(&file)?,
        Commands::CatFile { hash } => commands::cat_file::run(&hash)?,
        Commands::Add { path } => commands::add::run(&path)?,
        Commands::Commit {
            message,
            author_name,
            author_email,
        } => commands::commit::run(&message, author_name.as_deref(), author_email.as_deref())?,
        Commands::Log => commands::log::run()?,
        Commands::Status => commands::status::run()?,
        Commands::Branch { name } => commands::branch::run(name.as_deref())?,
        Commands::Switch { name } => commands::switch::run(&name)?,
    }

    Ok(())
}
