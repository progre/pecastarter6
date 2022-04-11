use tauri::api::dialog;

pub fn show_file_error_dialog(message: &str) {
    let none: Option<&tauri::Window> = None;
    dialog::blocking::message(none, "Fatal", message);
}
