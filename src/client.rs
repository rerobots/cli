// SCL <scott@rerobots.net>
// Copyright (C) 2021 rerobots, Inc.

use actix::prelude::*;

use openssl::ssl::{SslMethod, SslConnector};


struct ClientError {
    msg: String,
}
impl std::error::Error for ClientError {}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::fmt::Debug for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl ClientError {
    fn newbox<T, S>(msg: S) -> Result<T, Box<dyn std::error::Error>>
    where
        S: ToString
    {
        Err(Box::new(ClientError { msg: msg.to_string() }))
    }
}


pub fn api_search(query: Option<&str>, types: Option<&Vec<&str>>, token: Option<&str>) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let origin = match option_env!("REROBOTS_ORIGIN") {
        Some(u) => u,
        None => "https://api.rerobots.net"
    };
    let authheader = match token {
        Some(tok) => Some(format!("Bearer {}", tok)),
        None => match std::env::var_os("REROBOTS_API_TOKEN") {
            Some(tok) => Some(format!("Bearer {}", tok.into_string().unwrap())),
            None => None
        }
    };
    let mut path = "/deployments?info=t".to_string();
    if let Some(q) = query {
        path.push_str(format!("&q={}", q).as_str());
    }
    if let Some(t) = types {
        path.push_str(format!("&types={}", t.join(",")).as_str());
    }

    let url = format!("{}{}", origin, path);
    let connector = SslConnector::builder(SslMethod::tls())?.build();

    let mut sys = System::new("wclient");
    actix::SystemRunner::block_on(&mut sys, async move {
        let client = awc::Client::builder()
            .connector(awc::Connector::new().ssl(connector).finish());
        let client = match authheader {
            Some(hv) => client.header("Authorization", hv),
            None => client
        }.finish();
        let mut resp = client.get(url).send().await?;
        if resp.status() == 200 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            debug!("resp to GET {}: {}", path, serde_json::to_string(&payload)?);
            Ok(payload)
        } else if resp.status() == 400 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            ClientError::newbox(String::from(payload["error_message"].as_str().unwrap()))
        } else {
            ClientError::newbox(format!("server indicated error: {}", resp.status()))
        }
    })
}
