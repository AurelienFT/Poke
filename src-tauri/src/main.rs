// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use api::{Account, Captcha, CookieJar};
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
async fn login(username: &str, password: &str, captcha_data: String, state: tauri::State<'_, PokeState>) -> Result<String, String> {
  let account = Account::login(username.to_string(), password.to_string(), Captcha::Resolved(captcha_data, state.cookies.clone())).await;
  Ok(account.access_token)
}

#[tauri::command]
async fn get_captcha_data() -> Result<CaptchaData, String> {
  println!("Getting captcha data");
  let (website_key, data, _) = Account::get_captcha_data().await;
  Ok(CaptchaData {
    website_key,
    data
  })
}

#[tokio::main]
async fn main() {
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
