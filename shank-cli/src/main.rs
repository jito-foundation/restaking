use anyhow::{anyhow, Result};
use env_logger::Env;
use log::{debug, info};
use shank_idl::manifest::Manifest;
use shank_idl::{extract_idl, ParseIdlOpts};
use std::fs::File;
use std::io::Write;

struct IdlConfiguration {
    program_id: String,
    name: &'static str,
    paths: Vec<&'static str>,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let crate_root = std::env::current_dir()?;

    let envs = envfile::EnvFile::new(crate_root.join("config").join("program.env"))?;
    let restaking_program_id = envs
        .get("RESTAKING_PROGRAM_ID")
        .ok_or(anyhow!("RESTAKING_PROGRAM_ID not found"))?
        .to_string();
    let vault_program_id = envs
        .get("VAULT_PROGRAM_ID")
        .ok_or(anyhow!("VAULT_PROGRAM_ID not found"))?
        .to_string();

    let idl_configs = vec![
        IdlConfiguration {
            program_id: restaking_program_id,
            name: "jito_restaking",
            paths: vec![
                "restaking_sdk",
                "restaking_core",
                "restaking_program",
                "bytemuck",
                "core",
            ],
        },
        IdlConfiguration {
            program_id: vault_program_id.to_string(),
            name: "jito_vault",
            paths: vec![
                "vault_sdk",
                "vault_core",
                "vault_program",
                "bytemuck",
                "core",
            ],
        },
    ];

    let crate_root = std::env::current_dir().unwrap();
    let out_dir = crate_root.join("idl");
    for idl in idl_configs {
        let mut idls = Vec::new();
        for path in idl.paths {
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
                .ok_or(anyhow!("Program needs to be a lib"))?;
            debug!("lib_rel_path: {:?}", lib_rel_path);
            let lib_full_path_str = crate_root.join(path).join(lib_rel_path);
            let lib_full_path = lib_full_path_str.to_str().ok_or(anyhow!("Invalid Path"))?;
            debug!("lib_full_path: {:?}", lib_full_path);
            // Extract IDL and convert to JSON
            let opts = ParseIdlOpts {
                program_address_override: Some(idl.program_id.to_string()),
                ..ParseIdlOpts::default()
            };
            let idl =
                extract_idl(lib_full_path, opts)?.ok_or(anyhow!("No IDL could be extracted"))?;
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
        accumulator.name = idl.name.to_string();

        let idl_json = accumulator.try_into_json()?;
        let mut idl_path = out_dir.join(idl.name);
        idl_path.set_extension("json");

        info!("Writing IDL to {:?}", idl_path);
        let mut idl_json_file = File::create(idl_path)?;
        idl_json_file.write_all(idl_json.as_bytes())?;
    }

    Ok(())
}

// pub fn idl(
//     out_dir: String,
//     out_filename: Option<String>,
//     crate_root: Option<String>,
//     program_id: Option<String>,
// ) -> Result<()> {
//     // Resolve input and output directories
//     let crate_root = try_resolve_path(crate_root, "crate_root")?;
//     let out_dir = try_resolve_path(Some(out_dir), "out_dir")?;
//     fs::create_dir_all(&out_dir)
//         .map_err(|err| format_err!("Unable to create out_dir ({}), {}", &out_dir.display(), err))?;
//
//     // Resolve info about lib for which we generate IDL
//     let cargo_toml = crate_root.join("Cargo.toml");
//     if !cargo_toml.exists() {
//         return Err(anyhow!(
//             "Did not find Cargo.toml at the path: {}",
//             crate_root.display()
//         ));
//     }
//     let manifest = Manifest::from_path(&cargo_toml)?;
//     let lib_rel_path = manifest
//         .lib_rel_path()
//         .ok_or(anyhow!("Program needs to be a lib"))?;
//
//     let lib_full_path_str = crate_root.join(lib_rel_path);
//     let lib_full_path = lib_full_path_str.to_str().ok_or(anyhow!("Invalid Path"))?;
//
//     // Extract IDL and convert to JSON
//     let opts = ParseIdlOpts {
//         program_address_override: program_id,
//         ..ParseIdlOpts::default()
//     };
//     let idl = extract_idl(lib_full_path, opts)?.ok_or(anyhow!("No IDL could be extracted"))?;
//     let idl_json = idl.try_into_json()?;
//
//     // Write to JSON file
//     let out_filename = if let Some(out_filename) = out_filename {
//         out_filename
//     } else {
//         format!("{}.json", manifest.lib_name()?)
//     };
//     let idl_json_path = out_dir.join(out_filename);
//     let mut idl_json_file = File::create(&idl_json_path)?;
//     info!("Writing IDL to {}", &idl_json_path.display());
//
//     idl_json_file.write_all(idl_json.as_bytes())?;
//
//     Ok(())
// }
