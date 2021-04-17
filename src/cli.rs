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
    fn new_std(err: Box<dyn std::error::Error>, exitcode: i32) -> Result<(), CliError> {
        Err(CliError { msg: Some(format!("{}", err)), exitcode: exitcode })
    }
}


fn search_subcommand(matches: &clap::ArgMatches) -> Result<(), CliError> {
    let query = matches.value_of("query");
    let payload = match client::api_search(query, Some(&vec!["!user_provided"]), None) {
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

    if matches.is_present("version") {
        println!(crate_version!());
    } else if let Some(_) = matches.subcommand_matches("version") {
        println!(crate_version!());
    } else if let Some(matches) = matches.subcommand_matches("search") {
        return search_subcommand(matches);
    } else {
        println!("No command given. Try `hardshare -h`");
    }

    Ok(())
}
