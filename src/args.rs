use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value = "db_me")]
    pub tg_database_directory: String,
    #[arg(short, long, default_value = "db.sqlite")]
    pub database_path: PathBuf,
    #[arg(short, long, default_value = "alter-mistral")]
    pub model_name: String,
}
