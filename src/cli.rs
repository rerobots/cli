// SCL <scott@rerobots.net>
// Copyright (C) 2021 rerobots, Inc.

use std::fs::{File, OpenOptions};
use std::io::prelude::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use clap::{Arg, SubCommand};

use crate::client;


pub struct CliError {
    pub msg: Option<String>,
    pub exitcode: i32,
}
impl std::error::Error for CliError {}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.msg {
            Some(m) => write!(f, "{}", m),
            None => write!(f, "")
        }
    }
}

impl std::fmt::Debug for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.msg {
            Some(m) => write!(f, "{}", m),
            None => write!(f, "")
        }
    }
}

impl CliError {
    fn new<S: ToString>(msg: S, exitcode: i32) -> Result<(), CliError> {
        Err(CliError { msg: Some(msg.to_string()), exitcode: exitcode })
    }

    fn new_std(err: Box<dyn std::error::Error>, exitcode: i32) -> Result<(), CliError> {
        Err(CliError { msg: Some(format!("{}", err)), exitcode: exitcode })
    }

    fn new_stdio(err: std::io::Error, exitcode: i32) -> Result<(), CliError> {
        Err(CliError { msg: Some(format!("{}", err)), exitcode: exitcode })
    }
}


fn search_subcommand(matches: &clap::ArgMatches, api_token: Option<String>) -> Result<(), CliError> {
    let query = matches.value_of("query");
    let type_constraint = if matches.is_present("with_user_provided") {
        None
    } else {
        Some(vec!["!user_provided"])
    };
    let payload = match client::api_search(query, type_constraint.as_ref(), api_token) {
        Ok(p) => p,
        Err(err) => return CliError::new_std(err, 1)
    };
    for wd in payload["workspace_deployments"].as_array().unwrap().iter() {
        let wd = wd.as_str().unwrap();
        let wtype = payload["info"][wd]["type"].as_str().unwrap();
        println!("{}    {}", wd, wtype);
    }
    Ok(())
}


fn list_subcommand(matches: &clap::ArgMatches, api_token: Option<String>) -> Result<(), CliError> {
    let payload = match client::api_instances(api_token) {
        Ok(p) => p,
        Err(err) => return CliError::new_std(err, 1)
    };
    for inst in payload["workspace_instances"].as_array().unwrap().iter() {
        let inst = inst.as_str().unwrap();
        println!("{}", inst);
    }
    Ok(())
}


fn info_subcommand(matches: &clap::ArgMatches, api_token: Option<String>) -> Result<(), CliError> {
    let instance_id = matches.value_of("instance_id");
    let payload = match client::api_instance_info(instance_id, api_token) {
        Ok(p) => p,
        Err(err) => return CliError::new_std(err, 1)
    };
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    Ok(())
}


fn terminate_subcommand(matches: &clap::ArgMatches, api_token: Option<String>) -> Result<(), CliError> {
    let instance_id = matches.value_of("instance_id");
    match client::api_terminate_instance(instance_id, api_token) {
        Ok(()) => Ok(()),
        Err(err) => return CliError::new_std(err, 1)
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
    let mut fp = OpenOptions::new().create_new(true).write(true).truncate(true).open(fname)?;
    user_only_perm(&mut fp)?;
    fp.write_all(secret_key.as_bytes())?;
    fp.sync_all()?;
    Ok(())
}


fn launch_subcommand(matches: &clap::ArgMatches, api_token: Option<String>) -> Result<(), CliError> {
    let wdid_or_wtype = matches.value_of("wdid_or_wtype").unwrap();

    let given_public_key = matches.is_present("public_key");
    let public_key = match matches.value_of("public_key") {
        Some(fname) => {
            if !std::path::Path::new(fname).exists() {
                return CliError::new(format!("Error: {} does not exist", fname), 1);
            }
            match std::fs::read_to_string(fname) {
                Ok(s) => Some(s.trim().to_string()),
                Err(err) => return CliError::new_stdio(err, 1)
            }
        },
        None => None
    };

    let secret_key_path = if !given_public_key {
        let path = match matches.value_of("secret_key") {
            Some(path) => path,
            None => "key.pem"
        };
        if std::path::Path::new(path).exists() {
            return CliError::new(format!("Error: {} already exists", path), 1);
        }
        Some(path)
    } else if matches.is_present("secret_key") {
        return CliError::new("Error: both --public-key and --secret-key given", 1);
    } else {
        None
    };

    let payload = match client::api_launch_instance(wdid_or_wtype, api_token, public_key) {
        Ok(p) => p,
        Err(err) => return CliError::new_std(err, 1)
    };
    println!("{}", payload["id"].as_str().unwrap());
    if !given_public_key {
        match write_secret_key(secret_key_path.unwrap(), payload["sshkey"].as_str().unwrap()) {
            Ok(()) => (),
            Err(err) => return CliError::new_std(err, 1)
        };
    }
    Ok(())
}


pub fn main() -> Result<(), CliError> {
    let app = clap::App::new("rerobots API command-line client")
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
        .arg(Arg::with_name("apitoken")
             .short("-t")
             .value_name("FILE")
             .help("plaintext file containing API token; with this flag, the REROBOTS_API_TOKEN environment variable is ignored"))
        .subcommand(SubCommand::with_name("search")
                    .about("Search for matching deployments. empty query implies show all existing workspace deployments")
                    .arg(Arg::with_name("query")
                         .value_name("QUERY"))
                    .arg(Arg::with_name("with_user_provided")
                         .long("include-user-provided")
                         .help("include user_provided workspace deployments in search")))
        .subcommand(SubCommand::with_name("list")
                    .about("List all instances by this user"))
        .subcommand(SubCommand::with_name("info")
                    .about("Print summary about instance")
                    .arg(Arg::with_name("instance_id")
                         .value_name("ID")))
        .subcommand(SubCommand::with_name("launch")
                    .about("Launch instance from specified workspace deployment or type")
                    .arg(Arg::with_name("wdid_or_wtype")
                         .value_name("ID")
                         .required(true)
                         .help("workspace type or deployment ID"))
                    .arg(Arg::with_name("public_key")
                         .long("public-key")
                         .value_name("FILE")
                         .help("path of public key to use; if not given, then a new key pair will be generated; this switch cannot be used with --secret-key"))
                    .arg(Arg::with_name("secret_key")
                         .long("secret-key")
                         .value_name("FILE")
                         .help("name of file in which to write new secret key (default key.pem)")))
        .subcommand(SubCommand::with_name("terminate")
                    .about("Terminate instance")
                    .arg(Arg::with_name("instance_id")
                         .value_name("ID")));

    let matches = app.get_matches();

    let default_loglevel = if matches.is_present("verbose") {
        "info"
    } else {
        "warn"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_loglevel)).init();

    let api_token = match matches.value_of("apitoken") {
        Some(fname) => {
            if !std::path::Path::new(fname).exists() {
                return CliError::new(format!("Error: {} does not exist", fname), 1);
            }
            match std::fs::read_to_string(fname) {
                Ok(s) => Some(s.trim().to_string()),
                Err(err) => return CliError::new_stdio(err, 1)
            }
        },
        None => None
    };

    if matches.is_present("version") {
        println!(crate_version!());
    } else if let Some(_) = matches.subcommand_matches("version") {
        println!(crate_version!());
    } else if let Some(matches) = matches.subcommand_matches("search") {
        return search_subcommand(matches, api_token);
    } else if let Some(matches) = matches.subcommand_matches("list") {
        return list_subcommand(matches, api_token);
    } else if let Some(matches) = matches.subcommand_matches("info") {
        return info_subcommand(matches, api_token);
    } else if let Some(matches) = matches.subcommand_matches("launch") {
        return launch_subcommand(matches, api_token);
    } else if let Some(matches) = matches.subcommand_matches("terminate") {
        return terminate_subcommand(matches, api_token);
    } else {
        println!("No command given. Try `hardshare -h`");
    }

    Ok(())
}
