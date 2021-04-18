// SCL <scott@rerobots.net>
// Copyright (C) 2021 rerobots, Inc.

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
    let payload = match client::api_search(query, Some(&vec!["!user_provided"]), api_token) {
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
                         .value_name("QUERY")));

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
    } else {
        println!("No command given. Try `hardshare -h`");
    }

    Ok(())
}
