use std::sync::mpsc;

mod bucket;
mod fax;

pub fn start_fax(data: FaxData, tx: mpsc::Sender<TxStatus>) {
    // Upload to bucket
    // Gen REQ
    // Sub to twilio
    // Monitor status
}

pub struct FaxData {
    fax_from: String,
    fax_to: String,
    media_path: String,
    creds: Credentials,
}

pub fn new_fax_data(fax_from: String, fax_to: String, media_path: String, creds: Credentials) -> FaxData {
    FaxData {
        fax_from,
        fax_to,
        media_path,
        creds
    }
}

pub struct Credentials {
    TwilioSID: String,
    TwilioSecret: String,
    TenantOCID: String,
    UserOCID: String
}

pub fn new_creds(TwilioSID: String, TwilioSecret: String, TenantOCID: String, UserOCID: String) -> Credentials {
    Credentials {
        TwilioSID,
        TwilioSecret,
        TenantOCID,
        UserOCID
    }
}

pub enum TxStatus {
    WaitUser,
    UploadFile(f32),
    GenPreauth,
    SubmitFax,
    FaxStatus,
    FaxError(String)
}
