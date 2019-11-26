use isahc::prelude::*;
use isahc::Error;
use isahc::http::{uri::Uri, header::HeaderMap};
use chrono::Utc;
use std::vec::Vec;
use std::io::prelude::*;
use std::fs::File;
use openssl::rsa::Rsa;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use openssl::hash::MessageDigest;
use openssl::base64::encode_block;
use crate::request::Credentials;

const BUCKET_NAME: &str = "bucket-fax";
const BUCKET_ENDPOINT: &str = "https://objectstorage.us-ashburn-1.oraclecloud.com";
const CERT_FILE: &str = "cert.pem";

//noinspection ALL
pub fn upload_object(creds: &Credentials, file_path: &str, file_name: &str) -> Result<(), String> {
    let namespace = get_namespace(creds).unwrap();
    println!("{:#?}", namespace);
    // let namespace = "idwvkbdltggo";
    // Upload file
    // let endpoint = format!("{}/n/{}/b/{}/o/{}", BUCKET_ENDPOINT, namespace, BUCKET_NAME, file_name).parse::<Uri>().unwrap();
    // let content_length = 1;
    // let request = Request::builder()
    //     .method("PUT")
    //     .uri(endpoint.clone())
    //     .header("content-length", content_length)
    //     .header("date", Utc::now().to_rfc2822())
    //     .header("host", endpoint.clone().host().unwrap())
    //     .header("(request-target)", format!("put {}", endpoint.query().unwrap()));
    Ok(())
}

fn gen_preauth() {

}

fn get_namespace(creds: &Credentials) -> Result<String, Error> {
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
        .body(())?
        .send();

    match request {
        // TODO: error handling
        Ok(result) => Ok(result.into_body().text().unwrap()),
        Err(error) => Err(error)
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
    println!("{:?}", signing_string);
    println!("{}", auth_header);

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
