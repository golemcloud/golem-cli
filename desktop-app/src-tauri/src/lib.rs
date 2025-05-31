use std::process::Command;
use std::sync::Mutex;
use tauri::State;

struct Storage {
    backend_url: Mutex<String>,
}
#[tauri::command]
fn update_backend_ip(new_ip: String, store: State<Storage>) -> Result<(), String> {
    // Save the new IP as a JSON value under the key "backend_ip"
    // store.
    println!("Updated backend IP to: {}", new_ip);
    *store.backend_url.lock().unwrap() = new_ip;
    Ok(())
}

#[tauri::command]
fn get_backend_ip(store: State<Storage>) -> Result<String, String> {
    // Retrieve the stored IP, if any, and convert it back to a string.
    Ok(store.backend_url.lock().unwrap().to_string())
}

// New function to create application using golem-cli
#[tauri::command]
fn create_golem_app(
    folder_path: String,
    app_name: String,
    language: String,
) -> Result<String, String> {
    // Determine the golem-cli path (use the one in PATH or a user-provided one)
    let golem_cli = "golem-cli"; // Can be extended to support a custom path

    println!("Creating a new {language} application named '{app_name}' in folder: {folder_path}");

    // Change to the selected directory and run the command
    let output = Command::new(golem_cli)
        .current_dir(&folder_path)
        .arg("app")
        .arg("new")
        .arg(&app_name)
        .arg(&language)
        .output()
        .map_err(|e| format!("Failed to execute golem-cli: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(format!(
            "Successfully created application: {}\n{}",
            app_name, stdout
        ))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Failed to create application: {}", stderr))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create an instance of the store.
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(Storage {
            backend_url: Mutex::from("http://localhost:9881".to_string()),
        })
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            update_backend_ip,
            get_backend_ip,
            create_golem_app
        ])
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
