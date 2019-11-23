mod bucket;
mod fax;

pub fn start_fax(data: FaxData) {
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

pub struct TwilioFax {
    From: String,
    To: String,
    MediaUrl: String,
    // StoreMedia: bool
}

pub struct Credentials {
    TwilioSID: String,
    TwilioSecret: String,
    TenantOCID: String,
    UserOCID: String
}

pub enum TxStatus {
    WaitUser,
    UploadFile(f32),
    GenPreauth,
    SubmitFax,
    FaxStatus,
    FaxError(String)
}
