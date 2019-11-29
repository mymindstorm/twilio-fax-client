use crate::request::{FaxData, Credentials};
use isahc::prelude::*;
use serde::{Deserialize, Serialize};
use openssl::base64::encode_block;

const TWILIO_ENDPOINT: &str = "https://fax.twilio.com/v1";

// Send fax request to twilio
pub fn sub_fax(data: &FaxData, media_uri: &str) -> Result<Fax, String> {
    let authorization = format!("{}:{}", data.creds.twilio_sid, data.creds.twilio_secret);
    let authorization = format!("Basic {}", encode_block(authorization.as_bytes()));
    
    let body = format!("To={}&From={}&MediaUrl={}&StoreMedia=false", basic_urlencode(&data.fax_to), basic_urlencode(&data.fax_from), basic_urlencode(media_uri));
    let request = Request::builder()
        .method("POST")
        .uri(format!("{}/Faxes", TWILIO_ENDPOINT))
        .header("Authorization", authorization)
        .body(body);

    let request = match request {
        Ok(res) => res,
        Err(err) => return Err(err.to_string())
    };

    match request.send() {
        Ok(res) => {
            if res.status().is_success() {
                let body = res.into_body().text().unwrap();
                let res: Fax = serde_json::from_str(body.as_str()).unwrap();
                Ok(res)
            } else {
                Err(res.into_body().text().unwrap())
            }
        },
        Err(err) => return Err(err.to_string())
    }
}

pub fn get_fax(creds: &Credentials, sid: &str) -> Result<Fax, String> {
    let authorization = format!("{}:{}", creds.twilio_sid, creds.twilio_secret);
    let authorization = format!("Basic {}", encode_block(authorization.as_bytes()));

    let request = Request::builder()
        .method("GET")
        .uri(format!("{}/Faxes/{}", TWILIO_ENDPOINT, sid))
        .header("Authorization", authorization)
        .body(());

        let request = match request {
            Ok(res) => res,
            Err(err) => return Err(err.to_string())
        };
    
        match request.send() {
            Ok(res) => {
                if res.status().is_success() {
                    let body = res.into_body().text().unwrap();
                    let res: Fax = serde_json::from_str(body.as_str()).unwrap();
                    Ok(res)
                } else {
                    Err(res.into_body().text().unwrap())
                }
            },
            Err(err) => return Err(err.to_string())
        }
}

fn basic_urlencode(input: &str) -> String {
    let mut input = String::from(input);
    input = input.replace(":", "%3A");
    input = input.replace("/", "%2F");
    input = input.replace("+", "%2B");
    input = input.replace("&", "%26");

    input
}

#[derive(Serialize, Deserialize)]
pub struct Fax {
    pub sid: String,
    account_sid: String,
    from: String,
    to: String,
    quality: String,
    media_sid: Option<String>,
    media_url: Option<String>,
    num_pages: Option<u32>,
    duration: Option<u32>,
    pub status: String,
    direction: String,
    api_version: String,
    price: Option<String>,
    price_unit: Option<String>,
    date_created: String,
    date_updated: String,
    links: FaxLinks,
    url: String
}

#[derive(Serialize, Deserialize)]
struct FaxLinks {
    media: String
}
