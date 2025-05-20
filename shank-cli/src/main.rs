use std::{fs::File, io::Write, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use env_logger::Env;
use log::{debug, info};
use shank_idl::{extract_idl, manifest::Manifest, ParseIdlOpts};

#[derive(Parser)]
#[command(author, version, about = "A CLI for managing shank", long_about = None)]
struct Cli {
    /// Path to the program_env file
    #[arg(long)]
    program_env_path: PathBuf,

    /// Path to the idl
    #[arg(long)]
    output_idl_path: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate IDL
    Generate(GenerateArgs),
}

#[derive(Args)]
struct GenerateArgs {
    /// Program id key in program.env
    #[arg(long)]
    program_id_key: String,

    /// IDL names
    #[arg(long)]
    idl_name: String,

    /// Module paths (core, program, sdk...)
    #[arg(long)]
    module_paths: Vec<String>,
}

struct IdlConfiguration {
    program_id: String,
    name: String,
    paths: Vec<String>,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args: Cli = Cli::parse();

    match args.command {
        Commands::Generate(generate_args) => {
            let envs = envfile::EnvFile::new(args.program_env_path)?;
            let program_id = envs
                .get(&generate_args.program_id_key)
                .ok_or_else(|| anyhow!("{} not found", generate_args.program_id_key))?
                .to_string();
            let idl_config = IdlConfiguration {
                program_id,
                name: generate_args.idl_name,
                paths: generate_args.module_paths,
            };

            let crate_root = std::env::current_dir().unwrap();
            let mut idls = Vec::new();
            for path in idl_config.paths.iter() {
                let cargo_toml = crate_root.join(path).join("Cargo.toml");
                if !cargo_toml.exists() {
                    return Err(anyhow!(
                        "Did not find Cargo.toml at the path: {}",
                        crate_root.display()
                    ));
                }
                let manifest = Manifest::from_path(&cargo_toml)?;
                debug!("manifest: {:?}", manifest);
                let lib_rel_path = manifest
                    .lib_rel_path()
                    .ok_or_else(|| anyhow!("Program needs to be a lib"))?;

                debug!("lib_rel_path: {:?}", lib_rel_path);
                let lib_full_path_str = crate_root.join(path).join(lib_rel_path);
                let lib_full_path = lib_full_path_str
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid Path"))?;
                debug!("lib_full_path: {:?}", lib_full_path);

                // Extract IDL and convert to JSON
                let opts = ParseIdlOpts {
                    program_address_override: Some(idl_config.program_id.to_string()),
                    ..ParseIdlOpts::default()
                };
                let idl = extract_idl(lib_full_path, opts)?
                    .ok_or_else(|| anyhow!("No IDL could be extracted"))?;
                idls.push(idl);
            }

            let mut accumulator = idls.pop().unwrap();
            for other_idls in idls {
                accumulator.constants.extend(other_idls.constants);
                accumulator.instructions.extend(other_idls.instructions);
                accumulator.accounts.extend(other_idls.accounts);
                accumulator.types.extend(other_idls.types);
                if let Some(events) = other_idls.events {
                    if let Some(accumulator_events) = &mut accumulator.events {
                        accumulator_events.extend(events);
                    } else {
                        accumulator.events = Some(events);
                    }
                }
                if let Some(errors) = other_idls.errors {
                    if let Some(accumulator_errors) = &mut accumulator.errors {
                        accumulator_errors.extend(errors);
                    } else {
                        accumulator.errors = Some(errors);
                    }
                }
            }
            accumulator.name = idl_config.name.to_string();

            let idl_json = accumulator.try_into_json()?;
            let mut idl_path = args.output_idl_path.join(idl_config.name);
            idl_path.set_extension("json");

            info!("Writing IDL to {:?}", idl_path);
            let mut idl_json_file = File::create(idl_path)?;
            idl_json_file.write_all(idl_json.as_bytes())?;
        }
    }

    Ok(())
}
