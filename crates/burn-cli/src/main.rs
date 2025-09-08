// Implementaiton TODO List
// - [] Generate basic project structure for pure Burn training of a model
//  - [x] New Rust lib using cargo
//  - [x] Modules
//      - [x] data
//      - [x] inference
//      - [x] model
//      - [x] training
//      - [x] lib w/ template
//  - [x] Polish up the module template contents
//  - [x] `bin` helper files
//  - [x] `crates` in Cargo.toml
// - [x] Polish output and feedback
// - [] Make hardcoded context options CLI flags

mod utils;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use console::{Emoji, style};
use minijinja::{Environment, context};
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process,
};
use toml_edit::{Array, DocumentMut, Value};

static FIRE: Emoji<'_, '_> = Emoji("🔥", "");

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Generate a new Burn project")]
    New {
        path: Option<PathBuf>,
        #[arg(short, long)]
        artifact_dir: Option<PathBuf>,
        #[arg(short, long, default_value = "ndarray")]
        backend: Backends,
        #[arg(short, long, default_value = "f32")]
        float_type: String,
        #[arg(short, long, default_value = "i32")]
        int_type: String,
    },
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
enum Backends {
    Candle,
    CandleCuda,
    CandleMetal,
    Cuda,
    Rocm,
    #[value(name = "ndarray")]
    NdArray,
    Tch,
    Vulkan,
    #[value(name = "webgpu")]
    WebGpu,
    Metal,
    Wgpu,
    Cpu,
}

impl Backends {
    /// Returns the Rust identifier for the corresponding Burn backend
    fn as_ident(&self) -> &str {
        match self {
            Self::Candle | Self::CandleCuda | Self::CandleMetal => "Candle",
            Self::Cuda => "Cuda",
            Self::Rocm => "Rocm",
            Self::NdArray => "NdArray",
            Self::Tch => "LibTorch",
            Self::Vulkan => "Vulkan",
            Self::WebGpu => "WebGpu",
            Self::Metal => "Metal",
            Self::Wgpu => "Wgpu",
            Self::Cpu => "Cpu",
        }
    }

    /// Returns the name for the corresponding burn feature
    /// to enable the backend.
    fn as_feature(&self) -> &str {
        match self {
            Self::Candle => "candle",
            Self::CandleCuda => "candle-cuda",
            Self::CandleMetal => "candle-metal",
            Self::Cuda => "cuda",
            Self::Rocm => "rocm",
            Self::NdArray => "ndarray",
            Self::Tch => "tch",
            Self::Vulkan => "vulkan",
            Self::WebGpu => "webgpu",
            Self::Metal => "metal",
            Self::Wgpu => "wgpu",
            Self::Cpu => "cpu",
        }
    }
}

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::New {
            path,
            artifact_dir,
            backend,
            float_type,
            int_type,
        } => {
            let project_path = match path {
                Some(path) => path,
                None => env::current_dir().expect("Failed to get current directory. Make sure the current directory exists and you have permissions to access it."),
            };
            println!("{}\n", style("Creating new Burn project...").bold());

            if let Err(err) = generate_project_structure(&project_path) {
                eprintln!("{err}");
                return;
            }

            let artifact_dir = match artifact_dir {
                Some(artifact_dir) => artifact_dir,
                None => {
                    let project_dir = match project_path.file_name() {
                        Some(project_dir) => project_dir,
                        None => {
                            eprintln!(
                                "{}",
                                style(format!(
                                    "{} is not a valid directory path.",
                                    project_path.display()
                                ))
                                .red()
                            );
                            return;
                        }
                    };

                    let mut artifact_dir = PathBuf::from("/tmp");
                    artifact_dir.push(project_dir);

                    artifact_dir
                }
            };

            if let Err(err) = generate_files(
                &project_path,
                &artifact_dir,
                backend,
                &float_type,
                &int_type,
            ) {
                eprintln!("{err}");
                return;
            }

            println!(
                "\n{} Burn project created at: {}",
                FIRE,
                style(project_path.display()).bold()
            );
        }
    }
}

fn generate_project_structure(project_path: &Path) -> anyhow::Result<()> {
    println!("[1/3] {}", style("Checking installed tools").bold());
    print!("  {}", style("cargo").dim());
    if !utils::check_cargo()? {
        return Err(anyhow!(
            "Could not find cargo insalled on the system. See https://www.rust-lang.org/tools/install for install instructions."
        ));
    }
    println!("{}", style("...done!").bold().green());

    println!("[2/3] {}", style("Creating new rust project").bold());
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

fn generate_files(
    project_path: &Path,
    artifact_dir: &Path,
    backend: Backends,
    float_type: &str,
    int_type: &str,
) -> anyhow::Result<()> {
    println!("[3/3] {}", style("Generating project files").bold());

    // Update Cargo.toml
    print!("  {}", style("adding dependencies").dim());
    let cargo_toml_path = project_path.join("Cargo.toml");
    let mut cargo_toml = fs::read_to_string(&cargo_toml_path)?.parse::<DocumentMut>()?;
    cargo_toml["dependencies"] = toml_edit::table();
    cargo_toml["dependencies"]
        .as_table_mut()
        .unwrap()
        .set_position(1);
    cargo_toml["dependencies"]["burn"]["version"] = toml_edit::value("0.18.0");
    let mut features = Array::new();
    features.push("std");
    features.push("tui");
    features.push("train");
    features.push("fusion");
    features.push(backend.as_feature());
    cargo_toml["dependencies"]["burn"]["features"] = toml_edit::value(Value::Array(features));
    cargo_toml["dependencies"]["burn"]["default-features"] = toml_edit::value(false);
    let mut cargo_toml_file = File::create(&cargo_toml_path)?;
    write!(cargo_toml_file, "{cargo_toml}")?;
    println!("{}", style("...done!").bold().green());

    let mut env = Environment::new();
    minijinja_embed::load_templates!(&mut env);

    // data.rs
    print!("  {}", style("generating src/data.rs").dim());
    let data_template = env.get_template("data.rs.jinja")?;
    let context = context! {
        model_name => "Mnist",
    };
    let content = data_template.render(context!(context))?;

    let mut data_file = File::create(project_path.join("src/data.rs"))?;
    write!(data_file, "{content}")?;
    println!("{}", style("...done!").bold().green());

    // inference.rs
    print!("  {}", style("generating src/inference.rs").dim());
    let inference_template = env.get_template("inference.rs.jinja")?;
    let context = context! {
        model_name => "Mnist",
    };
    let content = inference_template.render(context!(context))?;

    let mut inference_file = File::create(project_path.join("src/inference.rs"))?;
    write!(inference_file, "{content}")?;
    println!("{}", style("...done!").bold().green());

    // model.rs
    print!("  {}", style("generating src/model.rs").dim());
    let model_template = env.get_template("model.rs.jinja")?;
    let context = context! {};
    let content = model_template.render(context!(context))?;

    let mut model_file = File::create(project_path.join("src/model.rs"))?;
    write!(model_file, "{content}")?;
    println!("{}", style("...done!").bold().green());

    // training.rs
    print!("  {}", style("generating src/training.rs").dim());
    let training_template = env.get_template("training.rs.jinja")?;
    let context = context! {
        model_name => "Mnist",
    };
    let content = training_template.render(context!(context))?;

    let mut training_file = File::create(project_path.join("src/training.rs"))?;
    write!(training_file, "{content}")?;
    println!("{}", style("...done!").bold().green());

    // lib.rs
    print!("  {}", style("generating src/lib.rs").dim());
    let lib_template = env.get_template("lib.rs.jinja")?;
    let context = context! {};
    let content = lib_template.render(context!(context))?;

    let mut lib_file = File::create(project_path.join("src/lib.rs"))?;
    write!(lib_file, "{content}")?;
    println!("{}", style("...done!").bold().green());

    let project_name = match project_path.file_name() {
        Some(file_name) => match file_name.to_str() {
            Some(file_name) => file_name,
            None => {
                return Err(anyhow!(
                    "{} is an invalid project path. The directory name is not a valid string",
                    project_path.display()
                ));
            }
        },
        None => {
            return Err(anyhow!(
                "{} is an invalid project path. The path must end in a directory name",
                project_path.display()
            ));
        }
    };

    // main.rs - inference
    print!("  {}", style("generating src/main.rs").dim());
    let main_template = env.get_template("main.rs.jinja")?;
    let context = context! {
        project_name => project_name,
        backend => backend.as_ident(),
        artifact_dir => artifact_dir.to_str(),
        float_type => float_type,
        int_type => int_type,
    };
    let content = main_template.render(context!(context))?;

    let mut main_file = File::create(project_path.join("src/main.rs"))?;
    write!(main_file, "{content}")?;
    println!("{}", style("...done!").bold().green());

    // bin/train.rs
    print!("  {}", style("generating src/bin/train.rs").dim());
    let train_template = env.get_template("bin/train.rs.jinja")?;
    let context = context! {
        project_name => project_name,
        backend => backend.as_ident(),
        artifact_dir => artifact_dir.to_str(),
        float_type => float_type,
        int_type => int_type,
    };
    let content = train_template.render(context!(context))?;

    let train_file_path = project_path.join("src/bin/train.rs");
    fs::create_dir_all(train_file_path.parent().ok_or_else(|| {
        anyhow!(
            "Invalid parent path for train file {:?}",
            train_file_path.parent()
        )
    })?)?;
    let mut train_file = File::create(train_file_path)?;
    write!(train_file, "{content}")?;
    println!("{}", style("...done!").bold().green());

    Ok(())
}
