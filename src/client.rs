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

use std::collections::HashMap;

use actix::prelude::*;

use chrono::{TimeZone, Utc};

use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::ssl::{SslConnector, SslMethod};

use jwt::algorithm::openssl::PKeyWithDigest;
use jwt::VerifyWithKey;
use jwt::{Claims, Header, Token};


// TODO: this should eventually be placed in a public key store
#[cfg(not(test))]
const PUBLIC_KEY: &[u8] = include_bytes!("../keys/public.pem");

#[cfg(test)]
const PUBLIC_KEY: &[u8] = include_bytes!("../tests/keys/public.pem");


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
        S: ToString,
    {
        Err(Box::new(ClientError {
            msg: msg.to_string(),
        }))
    }
}


#[cfg(not(test))]
fn get_origin() -> String {
    option_env!("REROBOTS_ORIGIN")
        .unwrap_or("https://api.rerobots.net")
        .to_string()
}

#[cfg(test)]
fn get_origin() -> String {
    mockito::server_url()
}


/// API token
///
/// Manage yours at <https://rerobots.net/tokens>.
/// Learn more at <https://docs.rerobots.net/web/making-and-revoking-api-tokens>.
#[derive(Debug)]
pub struct TokenClaims {
    /// username
    pub subject: String,

    /// organization scope, if any
    pub organization: Option<String>,

    /// date after which this token is not valid
    pub expiration: Option<u64>,
}

impl TokenClaims {
    /// Attempt to parse raw string as API token.
    pub fn new(api_token: &str) -> Result<TokenClaims, &str> {
        let alg = PKeyWithDigest {
            digest: MessageDigest::sha256(),
            key: PKey::public_key_from_pem(PUBLIC_KEY).unwrap(),
        };

        let result: Result<Token<Header, Claims, _>, jwt::error::Error> =
            api_token.verify_with_key(&alg);
        let parsed_tok = match result {
            Ok(tok) => tok,
            Err(err) => match err {
                jwt::error::Error::InvalidSignature => return Err("not a valid signature"),
                _ => return Err("unknown error"),
            },
        };
        let claims = parsed_tok.claims();
        let subject = claims.registered.subject.as_ref().unwrap();
        let expiration = claims.registered.expiration;
        let organization = if claims.private.contains_key("org") {
            Some(claims.private["org"].as_str().unwrap().into())
        } else {
            None
        };
        Ok(TokenClaims {
            subject: subject.to_string(),
            expiration,
            organization,
        })
    }

    /// Is the expiration date of the API token in the past?
    /// Compares to [`std::time::SystemTime::now()`].
    pub fn is_expired(&self) -> bool {
        match self.expiration {
            Some(exp) => {
                let now = std::time::SystemTime::now();
                let utime = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                exp < utime
            }
            None => false,
        }
    }
}

impl std::fmt::Display for TokenClaims {
    /// Print token claims in a `KEY: VALUE` format.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "subject: {}", self.subject)?;
        match &self.organization {
            Some(org) => writeln!(f, "organization: {}", org)?,
            None => writeln!(f, "organization: (none)")?,
        };
        match self.expiration {
            Some(exp) => {
                write!(
                    f,
                    "expiration: {}",
                    Utc.timestamp_opt(exp as i64, 0).unwrap()
                )
            }
            None => write!(f, "expiration: (never)"),
        }
    }
}


fn create_client(token: Option<String>) -> Result<awc::Client, Box<dyn std::error::Error>> {
    let authheader = match token {
        Some(tok) => Some(format!("Bearer {}", tok)),
        None => std::env::var_os("REROBOTS_API_TOKEN")
            .map(|tok| format!("Bearer {}", tok.into_string().unwrap())),
    };

    let connector = SslConnector::builder(SslMethod::tls())?.build();
    let client = awc::Client::builder().connector(awc::Connector::new().ssl(connector).finish());
    Ok(match authheader {
        Some(hv) => client.header("Authorization", hv),
        None => client,
    }
    .finish())
}


pub fn api_search(
    query: Option<&str>,
    types: Option<&Vec<&str>>,
    token: Option<String>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
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


pub fn api_instances(
    token: Option<String>,
    include_terminated: bool,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut url = format!("{}/instances", get_origin());
    if include_terminated {
        url += "?include_terminated";
    }

    let mut sys = System::new("wclient");
    actix::SystemRunner::block_on(&mut sys, async {
        let client = create_client(token)?;
        let mut resp = client.get(url).send().await?;
        if resp.status() == 200 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            debug!(
                "resp to GET /instances: {}",
                serde_json::to_string(&payload)?
            );
            Ok(payload)
        } else if resp.status() == 400 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            ClientError::newbox(String::from(payload["error_message"].as_str().unwrap()))
        } else {
            ClientError::newbox(format!("server indicated error: {}", resp.status()))
        }
    })
}


fn select_instance<S: ToString>(
    instance_id: Option<S>,
    token: &Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let token = token.as_ref().cloned();
    match instance_id {
        Some(inid) => Ok(inid.to_string()),
        None => {
            let payload = api_instances(token, false)?;
            let instances = payload["workspace_instances"].as_array().unwrap();
            if instances.is_empty() {
                ClientError::newbox("no active instances")
            } else if instances.len() > 1 {
                ClientError::newbox("ambiguous command because more than one active instance")
            } else {
                Ok(instances[0].as_str().unwrap().to_string())
            }
        }
    }
}


pub fn api_instance_info<S: ToString>(
    instance_id: Option<S>,
    token: Option<String>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let instance_id = select_instance(instance_id, &token)?;
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


pub fn get_instance_sshkey<S: ToString>(
    instance_id: Option<S>,
    token: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let instance_id = select_instance(instance_id, &token)?;
    let path = format!("/instance/{}/sshkey", instance_id);
    let url = format!("{}{}", get_origin(), path);

    let mut sys = System::new("wclient");
    actix::SystemRunner::block_on(&mut sys, async move {
        let client = create_client(token)?;
        let mut resp = client.get(url).send().await?;
        if resp.status() == 200 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            debug!("resp to GET {}: {}", path, serde_json::to_string(&payload)?);
            Ok(payload["key"].as_str().unwrap().to_string())
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


pub fn api_wdeployment_info<S: std::fmt::Display>(
    wdeployment_id: S,
    token: Option<String>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let path = format!("/deployment/{}", wdeployment_id);
    let url = format!("{}{}", get_origin(), path);

    let mut sys = System::new("wclient");
    actix::SystemRunner::block_on(&mut sys, async move {
        let client = create_client(token)?;
        let req = client.get(url);
        let has_api_token = req.headers().contains_key("Authorization");
        let mut resp = req.send().await?;
        if resp.status() == 200 {
            let mut payload: serde_json::Value =
                serde_json::from_slice(resp.body().await?.as_ref())?;
            debug!("resp to GET {}: {}", path, serde_json::to_string(&payload)?);

            if has_api_token {
                let path = format!("{}/rules", path);
                let url = format!("{}{}", get_origin(), path);
                let mut resp = client.get(url).send().await?;
                if resp.status() == 200 {
                    let rules_payload: serde_json::Value =
                        serde_json::from_slice(resp.body().await?.as_ref())?;
                    debug!(
                        "resp to GET {}: {}",
                        path,
                        serde_json::to_string(&rules_payload)?
                    );
                    payload
                        .as_object_mut()
                        .unwrap()
                        .insert("cap".to_string(), rules_payload);
                } else if resp.status() == 400 {
                    let payload: serde_json::Value =
                        serde_json::from_slice(resp.body().await?.as_ref())?;
                    return ClientError::newbox(String::from(
                        payload["error_message"].as_str().unwrap(),
                    ));
                } else {
                    return ClientError::newbox(format!(
                        "server indicated error: {}",
                        resp.status()
                    ));
                }
            }

            Ok(payload)
        } else if resp.status() == 404 {
            ClientError::newbox("workspace deployment not found")
        } else if resp.status() == 400 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            ClientError::newbox(String::from(payload["error_message"].as_str().unwrap()))
        } else {
            ClientError::newbox(format!("server indicated error: {}", resp.status()))
        }
    })
}


pub fn api_terminate_instance(
    instance_id: Option<&str>,
    token: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let instance_id = select_instance(instance_id, &token)?;
    let path = format!("/terminate/{}", instance_id);
    let url = format!("{}{}", get_origin(), path);

    let mut sys = System::new("wclient");
    actix::SystemRunner::block_on(&mut sys, async move {
        let client = create_client(token)?;
        debug!("POST {}", path);
        let mut resp = client.post(url).send().await?;
        if resp.status() == 200 {
            Ok(())
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


pub fn api_launch_instance(
    wdid_or_wtype: &str,
    token: Option<String>,
    public_key: Option<String>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let td = std::time::Duration::new(10, 0);
    let path = format!("/new/{}", wdid_or_wtype);
    let url = format!("{}{}", get_origin(), path);

    let mut body = HashMap::new();
    if let Some(pk) = public_key {
        body.insert("sshkey", pk);
    }

    let mut sys = System::new("wclient");
    actix::SystemRunner::block_on(&mut sys, async move {
        let client_req = create_client(token)?.post(url).timeout(td);
        let mut resp = if !body.is_empty() {
            client_req.send_json(&body).await?
        } else {
            client_req.send().await?
        };
        if resp.status() == 200 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            debug!(
                "resp to POST {}: {}",
                path,
                serde_json::to_string(&payload)?
            );
            Ok(payload)
        } else if resp.status() == 400 {
            let payload: serde_json::Value = serde_json::from_slice(resp.body().await?.as_ref())?;
            ClientError::newbox(String::from(payload["error_message"].as_str().unwrap()))
        } else {
            ClientError::newbox(format!("server indicated error: {}", resp.status()))
        }
    })
}


#[cfg(test)]
mod tests {
    use mockito::mock;

    use super::api_search;
    use super::TokenClaims;

    #[test]
    fn empty_search() {
        let _m = mock("GET", "/deployments?info=t")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
  "info": {},
  "page_count": 1,
  "workspace_deployments": []
}"#,
            )
            .create();

        let res = api_search(None, None, None).unwrap();
        let wds = res["workspace_deployments"].as_array().unwrap();
        assert_eq!(wds.len(), 0)
    }


    #[test]
    fn search_with_1() {
        let _m = mock("GET", "/deployments?info=t")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
  "workspace_deployments": [
    "82051afa-b331-4b82-8bd4-9eea9ad78241"
  ],
  "page_count": 1,
  "info": {
    "82051afa-b331-4b82-8bd4-9eea9ad78241": {
      "type": "fixed_misty2",
      "type_version": 1,
      "supported_addons": [
        "cam",
        "mistyproxy",
        "py"
      ],
      "desc": "",
      "region": "us:cali",
      "icounter": 166,
      "created": "2021-07-17 03:37:44.284117",
      "queuelen": 0
    }
  }
}"#,
            )
            .create();

        let res = api_search(None, None, None).unwrap();
        let wds = res["workspace_deployments"].as_array().unwrap();
        assert_eq!(wds.len(), 1);
        assert_eq!(wds[0], "82051afa-b331-4b82-8bd4-9eea9ad78241");
    }


    #[test]
    fn detect_expired_token() {
        const EXPIRED_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ0ZXN0X3VzZXIiLCJpc3MiOiJyZXJvYm90cy5uZXQiLCJhdWQiOiJyZXJvYm90cy5uZXQiLCJleHAiOjE2Nzk2ODQ0MjQsIm5iZiI6MTY3OTU5ODAyNH0.R9Z4Y5tVHJiPTGfEjUljIGmzor4SAmUmdXyuvQBF2oc6QVHFfxGD-QnVaDfyB6Q2WbBiMWsvDgsI2X56t_-Cd7tZio-VQLL-AEwu1uTnOnt3aXwYB211M7b5ZEFrNN5BNS00FqnMpOQ5DfWKzYUqzvVV4gbxdPhLD2PUpMvT6-F-9NgtR_Z5VEeR-rzRI1-A0KYP9tWHh8GeEPb4D449wtcp-a-Hy6GHOFGGJupSkiB1aJ0T-b1CPlEDN9uwgEq4N_2hHMXwYiyc5Qpo5PABAB_BhM0CP43ca2M9n67om9oQZHqnkon_RWJDSjNAGCn8aZGSfKv0E2pahXfqjhWn6Eakb_R4VDNFBIy6xAl1i0NT-YDdF8-4kLA0sxIoLnk812LvmoifsFsmuv1cdSAAccYdyMjwyQDNCMjFCSJSR6pwZhpfsaUB1frTXWaFteA8yGf8bkL59i3yWherji7yfRY-aepVSH2Hjw_bHxVIPq3mulrhW0XI8qk6uPS1CB5F4Thqqvf_G-qIx0HJWAzF6zTkoAjhOa-BUIuxGo2w17fxK2RhzoOfMZWSfQqifKdCxhuGNOTpyD7nsK9OQP9_S1krOLSvavWuPfTV5GN-NhmRSAcg8Qcv1UkGguZaAFlHlGOzlw4PI_9qGhIxPj7t-PjHyETdH4IrdilpQkXgZuw";
        let tc = TokenClaims::new(EXPIRED_TOKEN).unwrap();
        assert!(tc.is_expired());
        assert_eq!(tc.subject, "test_user");
        assert_eq!(tc.organization, None);
        assert_eq!(tc.expiration.unwrap(), 1679684424);

        let mut tok = String::from(EXPIRED_TOKEN);
        tok.push('F');
        let error = TokenClaims::new(&tok).unwrap_err();
        assert_eq!(error, "not a valid signature");
    }
}
