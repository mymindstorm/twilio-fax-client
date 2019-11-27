use isahc::prelude::*;
use isahc::http::{Uri, HeaderMap};
use chrono::prelude::*;
use std::vec::Vec;
use std::io::prelude::*;
use std::fs::{File, metadata};
use openssl::rsa::Rsa;
use openssl::sha;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use openssl::hash::MessageDigest;
use openssl::base64::encode_block;
use crate::request::Credentials;
use serde::{Deserialize, Serialize};

const BUCKET_NAME: &str = "bucket-fax";
const BUCKET_ENDPOINT: &str = "https://objectstorage.us-ashburn-1.oraclecloud.com";
const CERT_FILE: &str = "cert.pem";

pub fn upload_object(creds: &Credentials, file_path: &str, file_name: &str) -> Result<(), String> {
    let namespace = get_namespace(creds);

    let namespace = match namespace {
        Ok(res) => res,
        Err(err) => return Err(err)
    };

    // Upload file
    let file_len = match metadata(file_path) {
        Ok(res) => res.len(),
        Err(err) => return Err(err.to_string())
    };
    let mut file = match File::open(file_path) {
        Ok(res) => res,
        Err(err) => return Err(err.to_string())
    };
    let mut file_buffer: Vec<u8> = Vec::new();
    match file.read_to_end(&mut file_buffer) {
        Ok(_) => {},
        Err(err) => return Err(err.to_string())
    };

    let endpoint = format!("{}/n/{}/b/{}/o/{}", BUCKET_ENDPOINT, namespace, BUCKET_NAME, file_name).parse::<Uri>().unwrap();
    let mut request = Request::builder();
    let headers = request.headers_mut().unwrap();
    headers.insert("date", Utc::now().to_rfc2822().replace("+0000", "GMT").parse().unwrap());
    headers.insert("host", endpoint.host().unwrap().parse().unwrap());
    headers.insert("content-length", file_len.to_string().parse().unwrap());
    let auth_header = sign_request(&headers, &endpoint, "put", creds);
    headers.insert("authorization", auth_header.parse().unwrap());

    let request = request.method("PUT")
        .uri(endpoint)
        .body(file_buffer);
    
    let request = match request {
        Ok(res) => res,
        Err(err) => return Err(err.to_string())
    };

    match request.send() {
        Ok(res) => {
            if res.status().is_success() {
                match res.into_body().text() {
                    Ok(res) => res,
                    Err(err) => return Err(err.to_string())
                }
            } else {
                return Err(res.into_body().text().unwrap_or_default())
            }
        },
        Err(err) => return Err(err.to_string())
    };

    Ok(())
}

pub fn gen_preauth(creds: &Credentials, file_name: &str) -> Result<PreauthenticatedRequest, String> {
    let namespace = get_namespace(creds);
    let namespace = match namespace {
        Ok(res) => res,
        Err(err) => return Err(err)
    };

    let endpoint = format!("{}/n/{}/b/{}/p/", BUCKET_ENDPOINT, namespace, BUCKET_NAME);
    let endpoint = endpoint.parse::<Uri>().unwrap();
    
    let auth_req_name = format!("twilio-req-{}", Utc::now().timestamp());
    let today = Utc::now().timestamp();
    let expiry = Utc.timestamp(today + 86400, 0);
    let body = CreatePreauthenticatedRequestDetails {
        name: auth_req_name,
        objectName: String::from(file_name),
        accessType: String::from("ObjectRead"),
        timeExpires: expiry.to_rfc3339()
    };
    let body = serde_json::to_string(&body).unwrap();

    let mut request = Request::builder();
    {
        let headers = request.headers_mut().unwrap();
        let time = Utc::now().to_rfc2822().replace("+0000", "GMT");
        headers.insert("date", time.parse().unwrap());
        headers.insert("host", endpoint.host().unwrap().parse().unwrap());
        headers.insert("content-type", "application/json".parse().unwrap());
        headers.insert("content-length", body.len().to_string().parse().unwrap());
        headers.insert("x-content-sha256", encode_block(&sha::sha256(body.as_bytes())).parse().unwrap());
        let auth_header = sign_request(&headers, &endpoint, "post", creds);
        headers.insert("authorization", auth_header.parse().unwrap());
    }

    let request = request.method("POST")
        .uri(endpoint)
        .body(body).unwrap()
        .send();

    match request {
        // TODO: error handling
        Ok(result) => {
            if result.status().is_success() {
                let result: PreauthenticatedRequest = serde_json::from_str(result.into_body().text().unwrap().as_str()).unwrap();
                Ok(result)
            } else {
                Err(result.into_body().text().unwrap())
            }
        },
        Err(error) => Err(error.to_string())
    }
}

fn get_namespace(creds: &Credentials) -> Result<String, String> {
    let endpoint = format!("{}/n/", BUCKET_ENDPOINT);
    let endpoint = endpoint.parse::<Uri>().unwrap();
    
    let mut request = Request::builder();
    {
        let headers = request.headers_mut().unwrap();
        let time = Utc::now().to_rfc2822().replace("+0000", "GMT");
        headers.insert("date", time.parse().unwrap());
        headers.insert("host", endpoint.host().unwrap().parse().unwrap());
        let auth_header = sign_request(&headers, &endpoint, "get", creds);
        headers.insert("authorization", auth_header.parse().unwrap());
    }

    let request = request.method("GET")
        .uri(endpoint)
        .body(()).unwrap()
        .send();

    match request {
        // TODO: error handling
        Ok(result) => {
            if result.status().is_success() {
                Ok(result.into_body().text().unwrap().replace("\"", ""))
            } else {
                Err(result.into_body().text().unwrap())
            }
        },
        Err(error) => Err(error.to_string())
    }
}

// Sign HTTP requests to oracle cloud and return Authorization
fn sign_request(headers: &HeaderMap, uri: &Uri, method: &str, creds: &Credentials) -> String {
    // Get signing string
    let mut signing_string = String::new();
    let mut auth_header = String::from(format!("Signature version=\"1\",keyId=\"{}/{}/{}\",algorithm=\"rsa-sha256\",headers=\"", creds.tenant_ocid, creds.user_ocid, creds.pub_cert));

    signing_string.push_str(&format!("(request-target): {} {}\n", method, match uri.path_and_query() { Some(val) => val.as_str(), None => ""}));
    auth_header.push_str("(request-target) ");

    for (key, val) in headers.iter() {
        signing_string.push_str(&format!("{}: {}\n", key, val.to_str().unwrap()));
        auth_header.push_str(&format!("{} ", key));
    }
    auth_header = String::from(auth_header.trim_end());
    auth_header.push_str("\"");
    signing_string.pop();

    #[cfg(debug_assertions)]
    println!("{}\n{}", signing_string, auth_header);
    // Sign
    let mut cert_file = File::open(CERT_FILE)
        .expect("Could not open cert.pem file.");
    let mut cert_buffer: Vec<u8> = Vec::new();
    cert_file.read_to_end(&mut cert_buffer).unwrap();
    let priv_key = Rsa::private_key_from_pem(cert_buffer.as_slice()).unwrap();
    let priv_key = PKey::from_rsa(priv_key).unwrap();
    let mut signer = Signer::new(MessageDigest::sha256(), &priv_key).unwrap();
    signer.update(signing_string.as_bytes()).unwrap();
    let signature = signer.sign_to_vec().unwrap();
    let signature = encode_block(signature.as_slice());
    auth_header = format!("{},signature=\"{}\"", auth_header, signature);

    auth_header
}

#[derive(Serialize, Deserialize)]
struct CreatePreauthenticatedRequestDetails {
    name: String,
    objectName: String,
    accessType: String,
    timeExpires: String
}

#[derive(Serialize, Deserialize)]
pub struct PreauthenticatedRequest {
    accessUri: String,
    id: String,
    name: String,
    accessType: String,
    timeCreated: String,
    timeExpires: String
}

impl PreauthenticatedRequest {
    pub fn get_uri(&self) -> String {
        format!("{}{}", BUCKET_ENDPOINT, &self.accessUri)
    }
}
