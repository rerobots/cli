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


fn get_origin() -> &'static str {
    match option_env!("REROBOTS_ORIGIN") {
        Some(u) => u,
        None => "https://api.rerobots.net"
    }
}


fn create_client(token: Option<String>) -> Result<awc::Client, Box<dyn std::error::Error>> {
    let authheader = match token {
        Some(tok) => Some(format!("Bearer {}", tok)),
        None => match std::env::var_os("REROBOTS_API_TOKEN") {
            Some(tok) => Some(format!("Bearer {}", tok.into_string().unwrap())),
            None => None
        }
    };

    let connector = SslConnector::builder(SslMethod::tls())?.build();
    let client = awc::Client::builder()
        .connector(awc::Connector::new().ssl(connector).finish());
    Ok(match authheader {
        Some(hv) => client.header("Authorization", hv),
        None => client
    }.finish())
}


pub fn api_search(query: Option<&str>, types: Option<&Vec<&str>>, token: Option<String>) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut path = "/deployments?info=t".to_string();
    if let Some(q) = query {
        path.push_str(format!("&q={}", q).as_str());
    }
    if let Some(t) = types {
        path.push_str(format!("&types={}", t.join(",")).as_str());
    }

    let url = format!("{}{}", get_origin(), path);

    let mut sys = System::new("wclient");
    actix::SystemRunner::block_on(&mut sys, async move {
        let client = create_client(token)?;
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


pub fn api_instances(token: Option<String>) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let url = format!("{}/instances", get_origin());

    let mut sys = System::new("wclient");
    actix::SystemRunner::block_on(&mut sys, async {
        let client = create_client(token)?;
        let mut resp = client.get(url).send().await?;
        if resp.status() == 200 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            debug!("resp to GET /instances: {}", serde_json::to_string(&payload)?);
            Ok(payload)
        } else if resp.status() == 400 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            ClientError::newbox(String::from(payload["error_message"].as_str().unwrap()))
        } else {
            ClientError::newbox(format!("server indicated error: {}", resp.status()))
        }
    })
}


pub fn api_instance_info(instance_id: Option<&str>, token: Option<String>) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let instance_id = match instance_id {
        Some(inid) => inid.to_string(),
        None => {
            let payload = api_instances(token.clone())?;
            let instances = payload["workspace_instances"].as_array().unwrap();
            if instances.len() == 0 {
                return ClientError::newbox("no active instances")
            } else if instances.len() > 1 {
                return ClientError::newbox("ambiguous command because more than one active instance")
            } else {
                instances[0].as_str().unwrap().to_string()
            }
        }
    };
    let path = format!("/instance/{}", instance_id);
    let url = format!("{}{}", get_origin(), path);

    let mut sys = System::new("wclient");
    actix::SystemRunner::block_on(&mut sys, async move {
        let client = create_client(token)?;
        let mut resp = client.get(url).send().await?;
        if resp.status() == 200 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            debug!("resp to GET {}: {}", path, serde_json::to_string(&payload)?);
            Ok(payload)
        } else if resp.status() == 404 {
            ClientError::newbox("instance not found")
        } else if resp.status() == 400 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            ClientError::newbox(String::from(payload["error_message"].as_str().unwrap()))
        } else {
            ClientError::newbox(format!("server indicated error: {}", resp.status()))
        }
    })
}
