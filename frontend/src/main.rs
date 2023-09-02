use std::{ops::Deref, time::Duration};

use gloo_utils::document;
use js_sys::Reflect;
use log::{info, error};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::prelude::*;

#[wasm_bindgen(module = "/public/glue.js")]
extern "C" {
    #[wasm_bindgen(js_name = invokeLogin, catch)]
    pub async fn login(username: String, password: String) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = invokeGetCaptchaData, catch)]
    pub async fn get_captcha_data() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = test)]
    pub fn test(data: String) -> JsValue;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CaptchaData {
    website_key: String,
    data: String,
}

#[function_component]
fn App() -> Html {
    let username = use_state_eq(|| "".to_string());
    let password = use_state_eq(|| "".to_string());
    let access_token = use_state(|| "".to_string());
    let captcha_data = use_state_eq::<Option<CaptchaData>, _>(|| None);
    spawn_local({
        let captcha_data = captcha_data.clone();
        async move {
            if (*captcha_data).is_some() {
                return;
            }
            match get_captcha_data().await {
                Ok(data) => {
                    info!("Got captcha data: {:?}", data);
                    let data: CaptchaData = serde_wasm_bindgen::from_value(data).unwrap();
                    captcha_data.set(Some(CaptchaData {
                        website_key: data.website_key,
                        data: data.data,
                    }));
                }
                Err(e) => {
                    let window = window().unwrap();
                    window
                        .alert_with_message(&format!("Error: {:?}", e))
                        .unwrap();
                }
            }
        }
    });
    use_effect_with_deps(
        move |captcha_data| {
            if let Err(e) = inject_script(captcha_data.clone()) {
                error!("{:?}", e);
            }
            || ()
        },
        (*captcha_data).clone(),
    );
    let message = (*access_token).clone();
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    let on_input_username = {
        let username = username.clone();
        Callback::from(move |event: InputEvent| {
            let value = event.data().unwrap();
            let current_value = (*username).clone();
            username.set(current_value + &value);
        })
    };

    let on_input_password = {
        let password = password.clone();
        Callback::from(move |event: InputEvent| {
            let value = event.data().unwrap();
            let current_value = (*password).clone();
            password.set(current_value + &value);
        })
    };

    let on_login = {
        Callback::from(move |_: MouseEvent| {
            perform_login(
                (*username).clone(),
                (*password).clone(),
                access_token.clone(),
            )
        })
    };

    html! {
        <div>
            <script src="https://hcaptcha.com/1/api.js?onload=GoogleRecaptchaLoaded" async=true defer=true />
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
            <input
                type="text"
                oninput={on_input_username}
            />
            <input
                type="password"
                oninput={on_input_password}
            />
            <button onclick={on_login}>{ "Connect" }</button>
            <div class="h-captcha" data-sitekey="019f1553-3845-481c-a6f5-5a60ccf6d830" data-theme="dark"></div>
        </div>
    }
}

fn perform_login(username: String, password: String, access_token: UseStateHandle<String>) {
    spawn_local(async move {
        // This will call our glue code all the way through to the tauri
        // back-end command and return the `Result<String, String>` as
        // `Result<JsValue, JsValue>`.
        match login(username, password).await {
            Ok(token) => {
                access_token.set(token.as_string().unwrap());
            }
            Err(e) => {
                let window = window().unwrap();
                window
                    .alert_with_message(&format!("Error: {:?}", e))
                    .unwrap();
            }
        }
    });
}

fn inject_script(captcha_data: Option<CaptchaData>)-> Result<(), JsValue> {
    if captcha_data.is_none() {
        return Ok(());
    }
    let google_loaded = Closure::wrap(Box::new(move |_| {
        if let Some(data) = captcha_data.clone() {
            test(data.data);
        } else {
            ()
        }
    }) as Box<dyn FnMut(JsValue)>);

    Reflect::set(
        &JsValue::from(web_sys::window().unwrap()),
        &JsValue::from("GoogleRecaptchaLoaded"),
        google_loaded.as_ref().unchecked_ref(),
    )?;
    google_loaded.forget();
    let script = document().create_element("script").unwrap();
    script.set_attribute("async", "true")?;
    script.set_attribute("defer", "true")?;
    let listener = Closure::wrap(Box::new(|_| {}) as Box<dyn FnMut(JsValue)>);
    let site_url = format!(
        "https://js.hcaptcha.com/1/api.js?hl=fr?onload=GoogleRecaptchaLoaded",
    );

    script.set_attribute("type", "text/javascript")?;
    let body = document()
        .body()
        .ok_or(JsValue::from_str("Can't find body"))?;
    body.append_child(&script)?;
    Ok(())
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
