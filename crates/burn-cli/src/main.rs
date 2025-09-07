// Implementaiton TODO List
// - [] Generate basic project structure for pure Burn training of a model
//  - [x] New Rust lib using cargo
//  - [] Modules
//      - [] data
//      - [] inference
//      - [] model
//      - [] training
//      - [] lib w/ template

mod utils;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use std::{
    env,
    path::{Path, PathBuf},
    process,
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    New { path: Option<PathBuf> },
}

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::New { path } => {
            let project_path = match path {
                Some(path) => path,
                None => env::current_dir().expect("Failed to get current directory. Make sure the current directory exists and you have permissions to access it."),
            };
            println!("Creating new project at {}", project_path.display());

            if let Err(err) = generate_project_structure(&project_path) {
                eprintln!("{err}");
            }
        }
    }
}

fn generate_project_structure(project_path: &Path) -> anyhow::Result<()> {
    if !utils::check_cargo()? {
        return Err(anyhow!(
            "Could not find cargo insalled on the system. See https://www.rust-lang.org/tools/install for install instructions."
        ));
    }

    let output = process::Command::new("cargo")
        .arg("new")
        .arg("--lib")
        .arg(format!("{}", project_path.display()))
        .output()?;

    println!("{}", String::from_utf8(output.stdout)?);

    Ok(())
}
