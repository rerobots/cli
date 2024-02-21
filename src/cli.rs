// Copyright (C) 2021 rerobots, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fs::{File, OpenOptions};
use std::io::prelude::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use clap::{Arg, SubCommand};

use rerobots::client;
use rerobots::client::TokenClaims;


#[derive(PartialEq)]
enum DefaultConfirmAnswer {
    Yes,
    No,
    None,
}


pub struct CliError {
    pub msg: Option<String>,
    pub exitcode: i32,
}
impl std::error::Error for CliError {}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.msg {
            Some(m) => write!(f, "{}", m),
            None => write!(f, ""),
        }
    }
}

impl std::fmt::Debug for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.msg {
            Some(m) => write!(f, "{}", m),
            None => write!(f, ""),
        }
    }
}

impl CliError {
    fn new<S: ToString>(msg: S, exitcode: i32) -> Result<(), CliError> {
        Err(CliError {
            msg: Some(msg.to_string()),
            exitcode,
        })
    }

    fn new_std(err: Box<dyn std::error::Error>, exitcode: i32) -> Result<(), CliError> {
        Err(CliError {
            msg: Some(format!("{}", err)),
            exitcode,
        })
    }

    fn new_stdio(err: std::io::Error, exitcode: i32) -> Result<(), CliError> {
        Err(CliError {
            msg: Some(format!("{}", err)),
            exitcode,
        })
    }

    fn newrc(exitcode: i32) -> Result<(), CliError> {
        Err(CliError {
            msg: None,
            exitcode,
        })
    }
}


#[derive(PartialEq, Debug)]
enum PrintingFormat {
    Default,
    Yaml,
    Json,
}


fn search_subcommand(
    matches: &clap::ArgMatches,
    api_token: Option<String>,
) -> Result<(), CliError> {
    let query = matches.value_of("query");
    let type_constraint = if matches.is_present("with_user_provided") {
        None
    } else {
        Some(vec!["!user_provided"])
    };
    let payload = match client::api_search(query, type_constraint.as_ref(), api_token) {
        Ok(p) => p,
        Err(err) => return CliError::new_std(err, 1),
    };
    for wd in payload["workspace_deployments"].as_array().unwrap().iter() {
        let wd = wd.as_str().unwrap();
        let wtype = payload["info"][wd]["type"].as_str().unwrap();
        println!("{}    {}", wd, wtype);
    }
    Ok(())
}


fn list_subcommand(matches: &clap::ArgMatches, api_token: Option<String>) -> Result<(), CliError> {
    let be_quiet = matches.is_present("quiet");
    let include_terminated = matches.is_present("include_terminated");
    let payload = match client::api_instances(api_token, include_terminated) {
        Ok(p) => p,
        Err(err) => return CliError::new_std(err, 1),
    };
    if !be_quiet {
        println!("instance\t\t\t\tworkspace deployment");
    }
    for (j, inst) in payload["workspace_instances"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let inst = inst.as_str().unwrap();
        if be_quiet {
            println!("{}", inst);
        } else {
            let wdeployment_id = &payload["workspace_deployments"].as_array().unwrap()[j]
                .as_str()
                .unwrap();
            println!("{}\t{}", inst, wdeployment_id);
        }
    }
    Ok(())
}


fn info_subcommand(
    matches: &clap::ArgMatches,
    api_token: Option<String>,
    pformat: PrintingFormat,
) -> Result<(), CliError> {
    let instance_id = matches.value_of("instance_id");
    let mut payload = match client::api_instance_info(instance_id, api_token) {
        Ok(p) => p,
        Err(err) => return CliError::new_std(err, 1),
    };
    payload["url"] = format!(
        "https://rerobots.net/instance/{}",
        payload["id"].as_str().unwrap()
    )
    .into();
    if pformat == PrintingFormat::Yaml {
        println!("{}", serde_yaml::to_string(&payload).unwrap());
    } else {
        // pformat == PrintingFormat::Json
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    }
    Ok(())
}


fn get_sshkey_subcommand(
    matches: &clap::ArgMatches,
    api_token: Option<String>,
    default_confirm: DefaultConfirmAnswer,
) -> Result<(), CliError> {
    let instance_id = matches.value_of("instance_id");

    let path = matches.value_of("secret_key_path").unwrap_or("key.pem");
    if std::path::Path::new(path).exists() && default_confirm != DefaultConfirmAnswer::Yes {
        if default_confirm == DefaultConfirmAnswer::No {
            return CliError::new(format!("Error: {} already exists", path), 1);
        }
        let prompt = format!(
            "Overwrite existing file at {} with new secret key? [y/N]",
            path
        );
        loop {
            print!("{} ", prompt);
            std::io::stdout().flush().unwrap();
            let mut choice = String::new();
            match std::io::stdin().read_line(&mut choice) {
                Ok(_) => {
                    choice.make_ascii_lowercase();
                    let choicel = choice.trim();
                    if choicel == "n" || choicel == "no" || choicel.is_empty() {
                        return CliError::newrc(1);
                    } else if choicel == "y" || choicel == "yes" {
                        break;
                    }
                }
                Err(err) => {
                    return CliError::new_stdio(err, 1);
                }
            }
        }
    }

    let key = match client::get_instance_sshkey(instance_id, api_token) {
        Ok(k) => k,
        Err(err) => return CliError::new_std(err, 1),
    };

    match write_secret_key(path, &key) {
        Ok(()) => Ok(()),
        Err(err) => CliError::new_std(err, 1),
    }
}


fn wdinfo_subcommand(
    matches: &clap::ArgMatches,
    api_token: Option<String>,
    pformat: PrintingFormat,
) -> Result<(), CliError> {
    let wdeployment_id = matches.value_of("wdeployment_id").unwrap();
    let payload = match client::api_wdeployment_info(wdeployment_id, api_token) {
        Ok(p) => p,
        Err(err) => return CliError::new_std(err, 1),
    };
    if pformat == PrintingFormat::Yaml {
        println!("{}", serde_yaml::to_string(&payload).unwrap());
    } else {
        // pformat == PrintingFormat::Json
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    }
    Ok(())
}


fn terminate_subcommand(
    matches: &clap::ArgMatches,
    api_token: Option<String>,
) -> Result<(), CliError> {
    let instance_id = matches.value_of("instance_id");
    match client::api_terminate_instance(instance_id, api_token) {
        Ok(()) => Ok(()),
        Err(err) => CliError::new_std(err, 1),
    }
}


fn isready_subcommand(
    matches: &clap::ArgMatches,
    api_token: Option<String>,
) -> Result<(), CliError> {
    let blocking = matches.is_present("blocking");
    let mut instance_id = matches.value_of("instance_id").map(|s| s.to_string());
    loop {
        let payload = match client::api_instance_info(instance_id.clone(), api_token.clone()) {
            Ok(p) => p,
            Err(err) => return CliError::new_std(err, 1),
        };
        let status = payload["status"].as_str().unwrap();
        if status == "READY" {
            return Ok(());
        } else if status != "INIT" || !blocking {
            return CliError::newrc(1);
        }
        if instance_id.is_none() {
            instance_id = Some(payload["id"].as_str().unwrap().to_string());
        }
        std::thread::sleep(std::time::Duration::new(1, 0));
    }
}


#[cfg(unix)]
fn user_only_perm(fp: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    let mut perm = fp.metadata()?.permissions();
    perm.set_mode(0o600);
    fp.set_permissions(perm)?;
    Ok(())
}

#[cfg(not(unix))]
fn user_only_perm(fp: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}


fn write_secret_key(fname: &str, secret_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut fp = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(fname)?;
    user_only_perm(&mut fp)?;
    fp.write_all(secret_key.as_bytes())?;
    fp.sync_all()?;
    Ok(())
}


fn decide_default_confirmation(matches: &clap::ArgMatches) -> DefaultConfirmAnswer {
    if matches.is_present("assume_no") {
        DefaultConfirmAnswer::No
    } else if matches.is_present("assume_yes") {
        DefaultConfirmAnswer::Yes
    } else {
        DefaultConfirmAnswer::None
    }
}


fn launch_subcommand(
    matches: &clap::ArgMatches,
    api_token: Option<String>,
) -> Result<(), CliError> {
    let wdid_or_wtype = matches.value_of("wdid_or_wtype").unwrap();

    let public_key = match matches.value_of("public_key") {
        Some(fname) => {
            if !std::path::Path::new(fname).exists() {
                return CliError::new(format!("Error: {} does not exist", fname), 1);
            }
            match std::fs::read_to_string(fname) {
                Ok(s) => Some(s.trim().to_string()),
                Err(err) => return CliError::new_stdio(err, 1),
            }
        }
        None => None,
    };

    let payload = match client::api_launch_instance(wdid_or_wtype, api_token, public_key) {
        Ok(p) => p,
        Err(err) => return CliError::new_std(err, 1),
    };
    println!("{}", payload["id"].as_str().unwrap());
    Ok(())
}


fn ssh_subcommand(matches: &clap::ArgMatches, api_token: Option<String>) -> Result<(), CliError> {
    let secret_key_path = "key.pem";
    let instance_id = matches.value_of("instance_id");
    let payload = match client::api_instance_info(instance_id, api_token) {
        Ok(p) => p,
        Err(err) => return CliError::new_std(err, 1),
    };
    let status = payload["status"].as_str().unwrap();
    if status != "READY" {
        return CliError::new("Error: instance is not READY", 1);
    }
    let username = "root";
    let ipv4 = payload["fwd"].as_object().unwrap()["ipv4"]
        .as_str()
        .unwrap();
    let port = payload["fwd"].as_object().unwrap()["port"]
        .as_u64()
        .unwrap();
    let args: Vec<&str> = match matches.values_of("ssh_args") {
        Some(v) => v.collect(),
        None => vec![],
    };

    let mut cmd = &mut std::process::Command::new("ssh");
    let mut gave_secretkey = false;
    for arg in args.iter() {
        if arg == &"-i" {
            gave_secretkey = true;
        }
        cmd = cmd.arg(arg);
    }
    if !gave_secretkey && std::path::Path::new(secret_key_path).exists() {
        cmd = cmd.arg("-i").arg(secret_key_path);
    }

    let status = match cmd
        .arg("-p")
        .arg(port.to_string())
        .arg(format!("{}@{}", username, ipv4))
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .status()
    {
        Ok(rc) => rc,
        Err(err) => return CliError::new_stdio(err, 1),
    };
    if status.success() {
        Ok(())
    } else {
        CliError::newrc(1)
    }
}


fn token_info_subcommand(
    matches: &clap::ArgMatches,
    api_token: Option<String>,
) -> Result<(), CliError> {
    let api_token = match matches.value_of("token_file") {
        Some(fname) => {
            if !std::path::Path::new(fname).exists() {
                return CliError::new(format!("Error: {} does not exist", fname), 1);
            }
            match std::fs::read_to_string(fname) {
                Ok(s) => Some(s.trim().to_string()),
                Err(err) => return CliError::new_stdio(err, 1),
            }
        }
        None => match api_token {
            Some(tok) => Some(tok),
            None => std::env::var_os("REROBOTS_API_TOKEN").map(|tok| tok.into_string().unwrap()),
        },
    };
    if api_token.is_none() {
        return CliError::new("No API token given", 1);
    }
    let api_token = api_token.unwrap();

    let tc = match TokenClaims::new(&api_token) {
        Ok(x) => x,
        Err(err) => return CliError::new(err, 1),
    };
    println!("{}", tc);
    if tc.is_expired() {
        println!("warning: This token is expired.");
        return CliError::newrc(1);
    }
    Ok(())
}


pub fn main() -> Result<(), CliError> {
    let app = clap::App::new("rerobots API command-line client").max_term_width(80)
        .subcommand(SubCommand::with_name("version")
                    .about("Prints version number and exits"))
        .arg(Arg::with_name("version")
             .short("V")
             .long("version")
             .help("Prints version number and exits"))
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Increases verboseness level of logs; ignored if RUST_LOG is defined"))
        .arg(Arg::with_name("printformat")
             .long("format")
             .value_name("FORMAT")
             .help("output formatting; options: YAML , JSON"))
        .arg(Arg::with_name("apitoken")
             .short("-t")
             .value_name("FILE")
             .help("plaintext file containing API token; with this flag, the REROBOTS_API_TOKEN environment variable is ignored"))
        .arg(Arg::with_name("assume_yes")
             .short("y")
             .help("assume \"yes\" for any questions required to execute the command; otherwise, interactive prompts will appear to confirm actions as needed"))
        .arg(Arg::with_name("assume_no")
             .short("n")
             .help("assume \"no\" for any questions required to execute the command; this can prevent destructive actions, e.g., overwriting a local file"))
        .subcommand(SubCommand::with_name("search")
                    .about("Search for matching deployments. empty query implies show all existing workspace deployments")
                    .arg(Arg::with_name("query")
                         .value_name("QUERY"))
                    .arg(Arg::with_name("with_user_provided")
                         .long("include-user-provided")
                         .help("include user_provided workspace deployments in search")))
        .subcommand(SubCommand::with_name("list")
                    .about("List all instances by this user")
                    .arg(Arg::with_name("quiet")
                         .short("q")
                         .long("quiet")
                         .help("Only display instance IDs"))
                    .arg(Arg::with_name("include_terminated")
                         .long("include-terminated")
                         .help("Include instances that are TERMINATED")))
        .subcommand(SubCommand::with_name("info")
                    .about("Print summary about instance")
                    .arg(Arg::with_name("instance_id")
                         .value_name("ID")))
        .subcommand(SubCommand::with_name("get-ssh-key")
                    .about("Get secret key for SSH access to instance")
                    .arg(Arg::with_name("instance_id")
                         .value_name("ID"))
                    .arg(Arg::with_name("secret_key_path")
                         .short("f")
                         .value_name("FILE")
                         .help("name of file in which to write new secret key (default key.pem)")))
        .subcommand(SubCommand::with_name("wdinfo")
                    .about("Print summary about workspace deployment")
                    .arg(Arg::with_name("wdeployment_id")
                         .value_name("ID")
                         .required(true)))
        .subcommand(SubCommand::with_name("launch")
                    .about("Launch instance from specified workspace deployment or type")
                    .arg(Arg::with_name("wdid_or_wtype")
                         .value_name("ID")
                         .required(true)
                         .help("workspace type or deployment ID"))
                    .arg(Arg::with_name("public_key")
                         .long("public-key")
                         .value_name("FILE")
                         .help("path of public key to use; if not given, then a new key pair will be generated")))
        .subcommand(SubCommand::with_name("terminate")
                    .about("Terminate instance")
                    .arg(Arg::with_name("instance_id")
                         .value_name("ID")))
        .subcommand(SubCommand::with_name("isready")
                    .about("Indicate whether instance is ready with exit code")
                    .arg(Arg::with_name("instance_id")
                         .value_name("ID"))
                    .arg(Arg::with_name("blocking")
                         .long("blocking")
                         .help("Do not return until instance is non-INIT")))
        .subcommand(SubCommand::with_name("ssh")
                    .about("Connect to instance host via ssh")
                    .arg(Arg::with_name("instance_id")
                         .value_name("ID"))
                    .arg(Arg::with_name("ssh_args")
                         .required(false)
                         .multiple(true)
                         .last(true)))
        .subcommand(SubCommand::with_name("token")
                    .about("Get information about an API token")
                    .arg(Arg::with_name("token_file")
                         .value_name("FILE")
                         .help("plaintext file containing API token; if not given, use REROBOTS_API_TOKEN environment variable or switch `-t`")));

    let matches = app.get_matches();

    let default_loglevel = if matches.is_present("verbose") {
        "info"
    } else {
        "warn"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_loglevel))
        .init();

    let pformat = match matches.value_of("printformat") {
        Some(given_pformat) => {
            let given_pformat_lower = given_pformat.to_lowercase();
            if given_pformat_lower == "json" {
                PrintingFormat::Json
            } else if given_pformat_lower == "yaml" {
                PrintingFormat::Yaml
            } else {
                return CliError::new(
                    format!("unrecognized format: {}", given_pformat).as_str(),
                    1,
                );
            }
        }
        None => PrintingFormat::Default,
    };

    let api_token = match matches.value_of("apitoken") {
        Some(fname) => {
            if !std::path::Path::new(fname).exists() {
                return CliError::new(format!("Error: {} does not exist", fname), 1);
            }
            match std::fs::read_to_string(fname) {
                Ok(s) => Some(s.trim().to_string()),
                Err(err) => return CliError::new_stdio(err, 1),
            }
        }
        None => None,
    };

    let default_confirm = decide_default_confirmation(&matches);

    if matches.is_present("version") || matches.subcommand_matches("version").is_some() {
        println!(crate_version!());
    } else if let Some(matches) = matches.subcommand_matches("search") {
        return search_subcommand(matches, api_token);
    } else if let Some(matches) = matches.subcommand_matches("list") {
        return list_subcommand(matches, api_token);
    } else if let Some(matches) = matches.subcommand_matches("info") {
        return info_subcommand(matches, api_token, pformat);
    } else if let Some(matches) = matches.subcommand_matches("get-ssh-key") {
        return get_sshkey_subcommand(matches, api_token, default_confirm);
    } else if let Some(matches) = matches.subcommand_matches("wdinfo") {
        return wdinfo_subcommand(matches, api_token, pformat);
    } else if let Some(matches) = matches.subcommand_matches("launch") {
        return launch_subcommand(matches, api_token);
    } else if let Some(matches) = matches.subcommand_matches("terminate") {
        return terminate_subcommand(matches, api_token);
    } else if let Some(matches) = matches.subcommand_matches("isready") {
        return isready_subcommand(matches, api_token);
    } else if let Some(matches) = matches.subcommand_matches("ssh") {
        return ssh_subcommand(matches, api_token);
    } else if let Some(matches) = matches.subcommand_matches("token") {
        return token_info_subcommand(matches, api_token);
    } else {
        println!("No command given. Try `rerobots -h`");
    }

    Ok(())
}
