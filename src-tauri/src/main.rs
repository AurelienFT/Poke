// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use api::Account;

mod api;


#[tauri::command]
async fn login(username: &str, password: &str) -> Result<String, String> {
  let account = Account::login(username.to_string(), password.to_string()).await;
  Ok(account.access_token)
}

fn main() {
  dotenv::dotenv().ok();
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![login])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
