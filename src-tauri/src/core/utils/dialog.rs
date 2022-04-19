use tauri::api::dialog;

pub fn show_dialog(message: &str) {
    let none: Option<&tauri::Window> = None;
    dialog::blocking::message(none, "Fatal", message);
}
