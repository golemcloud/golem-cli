use tauri::AppHandle;

use crate::services::store::AppStore;

// /// Sets the path to the golem-cli executable
// #[tauri::command]
// pub fn set_golem_cli_path(
//     app_handle: AppHandle, 
//     path: String,
// ) -> Result<(), String> {
//     // Validate the path exists
//     if !std::path::Path::new(&path).exists() {
//         return Err(format!("The specified path does not exist: {}", path));
//     }
    
//     // Save the path to settings
//     AppStore::save_golem_cli_path(&app_handle, &path)
// }
