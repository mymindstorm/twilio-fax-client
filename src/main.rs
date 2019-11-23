use imgui::*;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs::OpenOptions;
use std::convert::TryFrom;

mod support;
mod request;

fn main() {
    let system = support::init(file!());
    let mut state = UIState {
        form_send_to: ImString::with_capacity(20),
        form_file: ImString::with_capacity(261),
        conf_send_from: ImString::with_capacity(20),
        conf_twilio_sid: ImString::with_capacity(35),
        conf_twilio_token: ImString::with_capacity(200),
        status: Status::SendForm
    };

    let (mut conf_file, conf_data) = get_conf();

    match conf_data {
        Ok(data) => {
            state.conf_send_from = ImString::from(data.send_from);
            state.conf_twilio_sid = ImString::from(data.twilio_sid);
            state.conf_twilio_token = ImString::from(data.twilio_token);
        },
        Err(_err) => {
            state.conf_send_from.push_str("+1");
            state.status = Status::EditSettings;
        }
    }

    state.form_send_to.push_str("+1");

    system.main_loop(|_, ui| {
        match state.status {
            Status::SendForm => send_form(&ui, &mut state),
            Status::SubmitStatus => submit_status(&ui, &mut state),
            Status::EditSettings => edit_settings(&ui, &mut state, &mut conf_file),
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

fn send_form(ui: &Ui, ui_state: &mut UIState) {
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
                ui_state.status = Status::SubmitStatus; 
            }
        });
}

fn submit_status(ui: &Ui, ui_state: &mut UIState) {
    Window::new(im_str!("Sending Fax..."))
        .size([300.0, 200.0], Condition::FirstUseEver)
        .build(ui, || {
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
            ui.spacing();
            ui.separator();
            ui.spacing();
            if ui.small_button(im_str!("Save")) {
                let conf_data = ConfData {
                    send_from: String::from(ui_state.conf_send_from.to_str()),
                    twilio_sid: String::from(ui_state.conf_twilio_sid.to_str()),
                    twilio_token: String::from(ui_state.conf_twilio_token.to_str())
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
                ui_state.status = Status::SendForm;
            }
        });
}

struct UIState {
    form_send_to: ImString,
    form_file: ImString,
    conf_send_from: ImString,
    conf_twilio_sid: ImString,
    conf_twilio_token: ImString,
    status: Status
}

enum Status {
    EditSettings,
    SendForm,
    SubmitStatus
}

#[derive(Serialize, Deserialize)]
struct ConfData {
    send_from: String,
    twilio_sid: String,
    twilio_token: String
}
