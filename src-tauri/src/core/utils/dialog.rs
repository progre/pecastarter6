use native_dialog::{MessageDialog, MessageType};

pub fn show_dialog(message: &str) {
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title("Fatal")
        .set_text(message)
        .show_alert()
        .unwrap()
}

pub fn show_confirm(message: &str) -> bool {
    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("Confirm")
        .set_text(message)
        .show_confirm()
        .unwrap()
}
