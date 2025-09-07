// Implementaiton TODO List
// - [] Generate basic project structure for pure Burn training of a model
//  - [x] New Rust lib using cargo
//  - [x] Modules
//      - [x] data
//      - [x] inference
//      - [x] model
//      - [x] training
//      - [x] lib w/ template
//  - [] `bin` helper files
//  - [] `crates` in Cargo.toml
// - [] Polish output and feedback

mod utils;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use minijinja::{Environment, context};
use std::{
    env,
    fs::File,
    io::Write,
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
                return;
            }

            if let Err(err) = generate_files(&project_path) {
                eprintln!("{err}");
                return;
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

    if !output.status.success() {
        return Err(anyhow!("{}", String::from_utf8(output.stderr)?));
    }

    Ok(())
}

fn generate_files(project_path: &Path) -> anyhow::Result<()> {
    let mut env = Environment::new();
    minijinja_embed::load_templates!(&mut env);

    // data.rs
    let data_template = env.get_template("data.rs.jinja")?;
    let data_context = context! {};
    let content = data_template.render(context!(data_context))?;

    let mut data_file = File::create(project_path.join("src/data.rs"))?;
    write!(data_file, "{content}")?;

    // inference.rs
    let inference_template = env.get_template("inference.rs.jinja")?;
    let context = context! {
        item_name => "MnistItem",
    };
    let content = inference_template.render(context!(context))?;

    let mut inference_file = File::create(project_path.join("src/inference.rs"))?;
    write!(inference_file, "{content}")?;

    // model.rs
    let model_template = env.get_template("model.rs.jinja")?;
    let context = context! {};
    let content = model_template.render(context!(context))?;

    let mut model_file = File::create(project_path.join("src/model.rs"))?;
    write!(model_file, "{content}")?;

    // training.rs
    let training_template = env.get_template("training.rs.jinja")?;
    let context = context! {};
    let content = training_template.render(context!(context))?;

    let mut training_file = File::create(project_path.join("src/training.rs"))?;
    write!(training_file, "{content}")?;

    // lib.rs
    let lib_template = env.get_template("lib.rs.jinja")?;
    let context = context! {};
    let content = lib_template.render(context!(context))?;

    let mut lib_file = File::create(project_path.join("src/lib.rs"))?;
    write!(lib_file, "{content}")?;

    Ok(())
}
