use reqwest::{ClientBuilder, Url, header::{DATE, HOST, AUTHORIZATION, HeaderMap, HeaderName}};
use chrono::Utc;
use std::vec::Vec;
use std::io::prelude::*;
use std::fs::File;
use openssl::rsa::Rsa;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use openssl::hash::MessageDigest;
use openssl::base64::encode_block;

const BUCKET_NAME: &str = "bucket-fax";
const BUCKET_ENDPOINT: &str = "https://objectstorage.us-ashburn-1.oraclecloud.com";
const CERT_FILE: &str = "cert.pem";
const PUB_KEY_FINGERPRINT: &str = "69:95:52:3c:f3:ba:da:38:7b:78:48:6b:53:93:5c:e6";

pub fn upload_object(tenant_ocid: &str, user_ocid: &str, file_path: &str, file_name: &str) -> Result<(), String> {
    let namespace = get_namespace(tenant_ocid, user_ocid).unwrap();
    println!("{}", namespace);
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

fn get_namespace(tenant_ocid: &str, user_ocid: &str) -> Result<String, String> {
    let endpoint = format!("{}/n/", BUCKET_ENDPOINT);
    let endpoint = Url::parse(&endpoint).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(DATE, Utc::now().to_rfc2822().parse().unwrap());
    headers.insert(HOST, endpoint.host_str().unwrap().parse().unwrap());
    headers.insert(HeaderName::from_lowercase(b"(request-target)").unwrap(), format!("get {}", endpoint.query().unwrap()).parse().unwrap());
    let auth_header = sign_request(&headers, tenant_ocid, user_ocid);
    headers.insert(AUTHORIZATION, auth_header.parse().unwrap());
    let client = ClientBuilder::new();

    Ok(String::new())
}

fn sign_request(headers: &HeaderMap, tenant_ocid: &str, user_ocid: &str) -> String {
    // Get signing string
    let mut signing_string = String::new();
    let mut auth_header = String::from(format!("Signature version=\"1\",keyId=\"{}/{}/{}\",algorithm=\"rsa-sha256\",headers=\"", tenant_ocid, user_ocid, PUB_KEY_FINGERPRINT));

    for (key, val) in headers.iter() {
        signing_string.push_str(&format!("{}: {:?}\n", key, val));
        auth_header.push_str(&format!("{} ", key));
    }
    auth_header = String::from(auth_header.trim_end());
    auth_header.push_str("\"");
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
