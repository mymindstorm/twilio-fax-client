use std::sync::mpsc;
use chrono::Utc;

mod bucket;
mod fax;

pub fn start_fax(data: FaxData, tx: mpsc::Sender<TxStatus>) {
    // Upload to bucket
    tx.send(TxStatus::UploadFile).unwrap();
    let result = bucket::upload_object(&data.creds.tenant_ocid, &data.creds.user_ocid, &data.media_path, &data.media_name).unwrap();
    // Gen REQ
    // Sub to twilio
    // Monitor status
}

pub struct FaxData {
    fax_from: String,
    fax_to: String,
    media_path: String,
    media_name: String,
    creds: Credentials,
}

pub fn new_fax_data(fax_from: String, fax_to: String, media_path: String, creds: Credentials) -> FaxData {
    FaxData {
        fax_from,
        fax_to,
        media_path,
        media_name: format!("fax-{}.pdf", Utc::now().timestamp()),
        creds
    }
}

pub struct Credentials {
    twilio_sid: String,
    twilio_secret: String,
    tenant_ocid: String,
    user_ocid: String
}

pub fn new_creds(twilio_sid: String, twilio_secret: String, tenant_ocid: String, user_ocid: String) -> Credentials {
    Credentials {
        twilio_sid,
        twilio_secret,
        tenant_ocid,
        user_ocid
    }
}

pub enum TxStatus {
    WaitUser,
    UploadFile,
    GenPreauth,
    SubmitFax,
    FaxStatus,
    FaxError(String)
}
