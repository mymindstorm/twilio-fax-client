use http::{Request, Response};

const BUCKET_NAME: &str = "bucket-fax";
const BUCKET_ENDPOINT: &str = "https://objectstorage.us-ashburn-1.oraclecloud.com";

fn upload_object(namespace: &str, file_path: &str) {
    let request = Request::builder()
        .method("PUT")
        .uri("");
}

fn gen_preauth() {

}

fn sign_request() {

}
