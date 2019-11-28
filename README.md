# twilio-fax-client

My simple Twilio fax sending program. This is my first Rust program, so the code isn't the best. 

## Requirements

- Oracle Cloud object storage (request/bucket.rs)
- Twilio account

## Usage

#### Setting up Oracle Cloud

1. Create `cert.pem` in the directory you run the program. Follow the directions on [this page](https://docs.cloud.oracle.com/iaas/Content/API/Concepts/apisigningkey.htm) to generate a certificate.
2. Make a private bucket called `bucket-fax`.
3. Give the user you applied the certificate access to the bucket. (Identity => Policies)
    ```
    ALLOW GROUP <SOME_GROUP_HERE> to manage objects in tenancy where target.bucket.name = 'bucket-fax'
    ALLOW GROUP <SOME_GROUP_HERE> to manage buckets in tenancy where target.bucket.name = 'bucket-fax'
    ```

#### Run

```
cargo run
```
