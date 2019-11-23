use imgui::*;

mod support;
mod request;

fn main() {
    let system = support::init(file!());
    let mut state = State {
        form_send_to: ImString::with_capacity(20),
        form_send_from: ImString::with_capacity(20),
        form_file: ImString::with_capacity(261),
        form_ready: false
    };
    state.form_send_to.push_str("+1");
    state.form_send_from.push_str("+1");

    system.main_loop(|_, ui| {
        send_form(&ui, &mut state)
    });
}

fn send_form(ui: &Ui, state: &mut State) {
    Window::new(im_str!("Send Fax"))
            .size([300.0, 200.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.input_text(im_str!("Send to"), &mut state.form_send_to)
                    .chars_decimal(true)
                    .build();
                ui.input_text(im_str!("Send from"), &mut state.form_send_from)
                    .chars_decimal(true)
                    .build();
                ui.input_text(im_str!("File path"), &mut state.form_file)
                    .build();
                ui.spacing();
                ui.separator();
                ui.spacing();
                if ui.button(im_str!("Send Fax"), [70.0, 20.0]) {
                    state.form_ready = true;
                }
            });
}

struct State {
    form_send_to: ImString,
    form_send_from: ImString,
    form_file: ImString,
    form_ready: bool
}
