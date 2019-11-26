use imgui::*;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs::OpenOptions;
use std::convert::TryFrom;
use std::thread;
use std::sync::mpsc;
use std::rc::Rc;
use request::*;

mod support;
mod request;

fn main() {
    let system = support::init(file!());
    let (tx, rx): (mpsc::Sender<TxStatus>, mpsc::Receiver<TxStatus>) = mpsc::channel();
    let rx = Rc::new(rx);
    let mut state = UIState {
        form_send_to: ImString::with_capacity(20),
        form_file: ImString::with_capacity(261),
        conf_send_from: ImString::with_capacity(20),
        conf_twilio_sid: ImString::with_capacity(35),
        conf_twilio_token: ImString::with_capacity(200),
        conf_tenant_ocid: ImString::with_capacity(150),
        conf_user_ocid: ImString::with_capacity(150),
        conf_pub_cert: ImString::with_capacity(100),
        view: CurrentView::SendForm,
        fax_status: TxStatus::WaitUser,
    };

    let (mut conf_file, conf_data) = get_conf();

    match conf_data {
        Ok(data) => {
            state.conf_send_from = ImString::from(data.send_from);
            state.conf_twilio_sid = ImString::from(data.twilio_sid);
            state.conf_twilio_token = ImString::from(data.twilio_token);
            state.conf_tenant_ocid = ImString::from(data.tenant_ocid);
            state.conf_user_ocid = ImString::from(data.user_ocid);
            state.conf_pub_cert = ImString::from(data.pub_cert);
        },
        Err(_err) => {
            state.conf_send_from.push_str("+1");
            state.view = CurrentView::EditSettings;
        }
    }

    state.form_send_to.push_str("+1");

    system.main_loop(|_, ui| {
        match state.view {
            CurrentView::SendForm => send_form(&ui, &mut state, tx.clone()),
            CurrentView::SubmitStatus => submit_status(&ui, &mut state, Rc::clone(&rx)),
            CurrentView::EditSettings => edit_settings(&ui, &mut state, &mut conf_file),
        }
    });
}

fn get_conf() -> (std::fs::File, Result<ConfData, serde_json::Error>) {
    let mut conf_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("config.json")
        .expect("Couldn't open or create config.json");

    let mut conf_content = String::new();

    conf_file.read_to_string(&mut conf_content)
        .expect("Couldn't read from config.json");
    
    (conf_file, serde_json::from_str(&conf_content[..]))
}

fn send_form(ui: &Ui, ui_state: &mut UIState, tx: mpsc::Sender<TxStatus>) {
    Window::new(im_str!("Send Fax"))
        .size([300.0, 200.0], Condition::FirstUseEver)
        .build(ui, || {
            ui.input_text(im_str!("Send to"), &mut ui_state.form_send_to)
                .chars_decimal(true)
                .build();
            ui.input_text(im_str!("File path"), &mut ui_state.form_file)
                .build();
            ui.spacing();
            ui.separator();
            ui.spacing();
            if ui.small_button(im_str!("Send Fax")) {
                ui_state.view = CurrentView::SubmitStatus;
                let fax_data = new_fax_data(String::from(ui_state.conf_send_from.to_str()),
                    String::from(ui_state.form_send_to.to_str()),
                    String::from(ui_state.conf_send_from.to_str()),
                    new_creds(
                        String::from(ui_state.conf_twilio_sid.to_str()),
                        String::from(ui_state.conf_twilio_token.to_str()),
                        String::from(ui_state.conf_tenant_ocid.to_str()),
                        String::from(ui_state.conf_user_ocid.to_str()),
                        String::from(ui_state.conf_pub_cert.to_str())
                    ),
                );
                thread::spawn(move || {
                    start_fax(fax_data, tx);
                });
            }
        });
}

fn submit_status(ui: &Ui, ui_state: &mut UIState, rx: Rc<mpsc::Receiver<TxStatus>>) {
    Window::new(im_str!("Sending Fax..."))
        .size([300.0, 200.0], Condition::FirstUseEver)
        .build(ui, || {
            match rx.try_recv() {
                Ok(new_state) => ui_state.fax_status = new_state,
                Err(_) => { }
            }

            match &ui_state.fax_status {
                TxStatus::WaitUser => {},
                TxStatus::UploadFile => {
                    ui.text("Uploading PDF to bucket...");
                },
                TxStatus::GenPreauth => {
                    ui.text("Generating access URL for PDF...");
                },
                TxStatus::SubmitFax => {
                    ui.text("Submitting fax...");
                },
                TxStatus::FaxStatus => {
                    ui.text("API response here");
                },
                TxStatus::FaxError(error) => {
                    ui.text_wrapped(&ImString::new(error));
                    ui.spacing();
                    if ui.small_button(im_str!("Restart")) {
                        ui_state.view = CurrentView::SendForm;
                    }
                }
            }
        });

}

fn edit_settings(ui: &Ui, ui_state: &mut UIState, conf_file: &mut std::fs::File) {
    Window::new(im_str!("Setup"))
        .size([300.0, 200.0], Condition::FirstUseEver)
        .build(ui, || {
            ui.input_text(im_str!("Send from"), &mut ui_state.conf_send_from)
                .chars_decimal(true)
                .build();
            ui.input_text(im_str!("Twilio SID"), &mut ui_state.conf_twilio_sid)
                .build();
            ui.input_text(im_str!("Twilio token"), &mut ui_state.conf_twilio_token)
                .build();
            ui.input_text(im_str!("Tenant OCID"), &mut ui_state.conf_tenant_ocid)
                .build();
            ui.input_text(im_str!("User OCID"), &mut ui_state.conf_user_ocid)
                .build();
            ui.input_text(im_str!("Pub Cert Fingerprint"), &mut ui_state.conf_pub_cert)
                .build();
            ui.spacing();
            ui.separator();
            ui.spacing();
            if ui.small_button(im_str!("Save")) {
                let conf_data = ConfData {
                    send_from: String::from(ui_state.conf_send_from.to_str()),
                    twilio_sid: String::from(ui_state.conf_twilio_sid.to_str()),
                    twilio_token: String::from(ui_state.conf_twilio_token.to_str()),
                    tenant_ocid: String::from(ui_state.conf_tenant_ocid.to_str()),
                    user_ocid: String::from(ui_state.conf_user_ocid.to_str()),
                    pub_cert: String::from(ui_state.conf_pub_cert.to_str())
                };
                let conf_data = serde_json::to_value(conf_data)
                    .expect("Error serializing config data.");
                let conf_data = serde_json::to_vec_pretty(&conf_data)
                    .expect("Error serializing config data.");
                let conf_data_len = u64::try_from(conf_data.len())
                    .unwrap();

                conf_file.seek(SeekFrom::Start(0))
                    .unwrap();
                conf_file.write_all(&conf_data)
                    .expect("Error writing to config.json");
                conf_file.set_len(conf_data_len)
                    .unwrap();
                ui_state.view = CurrentView::SendForm;
            }
        });
}

struct UIState {
    form_send_to: ImString,
    form_file: ImString,
    conf_send_from: ImString,
    conf_twilio_sid: ImString,
    conf_twilio_token: ImString,
    conf_tenant_ocid: ImString,
    conf_user_ocid: ImString,
    conf_pub_cert: ImString,
    view: CurrentView,
    fax_status: TxStatus,
}

enum CurrentView {
    EditSettings,
    SendForm,
    SubmitStatus
}

#[derive(Serialize, Deserialize)]
struct ConfData {
    send_from: String,
    twilio_sid: String,
    twilio_token: String,
    tenant_ocid: String,
    user_ocid: String,
    pub_cert: String
}
