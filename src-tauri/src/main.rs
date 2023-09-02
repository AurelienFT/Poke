// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use api::{Account, Captcha, CookieJar};
use log::{info, warn};
use serde::{Serialize, Deserialize};

mod api;

#[derive(Serialize, Deserialize)]
struct CaptchaData {
  website_key: String,
  data: String
}

struct PokeState {
  captcha_data: CaptchaData,
  cookies: CookieJar
}

#[tauri::command]
async fn login(username: &str, password: &str, captcha: String, state: tauri::State<'_, PokeState>) -> Result<String, String> {
  let account = Account::login(username.to_string(), password.to_string(), Captcha::Resolved(captcha, state.cookies.clone())).await;
  Ok(account.access_token)
}

#[tauri::command]
async fn get_captcha_data(state: tauri::State<'_, PokeState>) -> Result<CaptchaData, String> {
  Ok(CaptchaData {
    website_key: state.captcha_data.website_key.clone(),
    data: state.captcha_data.data.clone()
  })
}

#[tokio::main]
async fn main() {
  env_logger::init();
  tauri::async_runtime::set(tokio::runtime::Handle::current());
  dotenv::dotenv().ok();
  let (website_key, data, cookies) = Account::get_captcha_data().await;
  tauri::Builder::default()
    .manage(PokeState {
      captcha_data: CaptchaData {
        website_key,
        data
      },
      cookies: cookies
    })
    .invoke_handler(tauri::generate_handler![login, get_captcha_data])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
