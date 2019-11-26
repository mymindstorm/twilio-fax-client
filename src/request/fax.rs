use crate::request::FaxData;
use isahc::prelude::*;
use serde::{Deserialize, Serialize};
use openssl::base64::encode_block;

const TWILIO_ENDPOINT: &str = "https://fax.twilio.com/v1";

// Send fax request to twilio
pub fn sub_fax(data: &FaxData, media_uri: &str) -> Result<Fax, String> {
    let authorization = format!("{}:{}", data.creds.twilio_sid, data.creds.twilio_secret);
    println!("{}", authorization);
    let authorization = format!("Basic {}", encode_block(authorization.as_bytes()));
    println!("{}", authorization);
    
    let body = format!("To={}\nFrom={}\nMediaUrl={}\nStoreMedia=false", data.fax_to, data.fax_from, media_uri); 
    
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
                let res: Fax = serde_json::from_str(res.into_body().text().unwrap().as_str()).unwrap();
                Ok(res)
            } else {
                Err(res.into_body().text().unwrap())
            }
        },
        Err(err) => return Err(err.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Fax {
    sid: String,
    accountSid: String,
    from: String,
    to: String,
    quality: String,
    mediaSid: String,
    mediaUrl: String,
    numPages: Option<u64>,
    duration: Option<u64>,
    status: String,
    direction: String,
    apiVersion: String,
    price: f32,
    priceUnit: String,
    dateCreated: String,
    dateUpdated: String,
    url: String
}
