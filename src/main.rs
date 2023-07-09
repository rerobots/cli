// Copyright (C) 2021 rerobots, Inc.

#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;

extern crate serde_json;
extern crate serde_yaml;

mod cli;
mod client;


fn main() {
    #[cfg(unix)]
    {
        openssl_probe::init_ssl_cert_env_vars();
    }
    match cli::main() {
        Ok(_) => std::process::exit(0),
        Err(err) => {
            if err.msg.is_some() {
                eprintln!("{}", err);
            }
            std::process::exit(err.exitcode);
        }
    }
}
