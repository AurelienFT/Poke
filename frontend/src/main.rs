use std::ops::Deref;

use log::info;
use wasm_bindgen::{JsValue, prelude::*};
use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use serde::{Serialize, Deserialize};

#[wasm_bindgen(module = "/public/glue.js")]
extern "C" {
    #[wasm_bindgen(js_name = invokeLogin, catch)]
    pub async fn login(username: String, password: String) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = invokeGetCaptchaData, catch)]
    pub async fn get_captcha_data() -> Result<JsValue, JsValue>;
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
                let data: CaptchaData = serde_wasm_bindgen::from_value(data).unwrap();
                info!("Captcha data: {:?}", data);
                captcha_data.set(Some(CaptchaData {
                    website_key: data.website_key,
                    data: data.data,
                }));
            }
            Err(e) => {
                info!("Error: {:?}", e);
                let window = window().unwrap();
                window
                    .alert_with_message(&format!("Error: {:?}", e))
                    .unwrap();
            }
        }
    }});
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
        Callback::from(
            move |event: InputEvent| {
                let value = event.data().unwrap();
                username.set(value);
            }
        )
    };

    let on_input_password = {
        let password = password.clone();
        Callback::from(
            move |event: InputEvent| {
                let value = event.data().unwrap();
                password.set(value);
            }
        )
    };

    let on_login = {
        Callback::from(
            move |_: MouseEvent| perform_login((*username).clone(), (*password).clone(), access_token.clone())
        )
    };

    let response_captcha = |response: String| {
        info!("Captcha response: {:?}", response);
    };

    html! {
        <div>
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
            {match (captcha_data.clone()).deref() {
                Some(data) => html! {
                <div>
                    <script src="https://js.hcaptcha.com/1/api.js" async=true defer=true />
                    <div class="h-captcha" data-sitekey={data.website_key.clone()} data-rqdata={data.data.clone()} data-theme="dark"></div>
                </div>
                },
                None => html! {
                    <p>{ "Loading captcha data..." }</p>
                }
            }}
            <p>{message}</p>
        </div>
    }
}

fn perform_login(username: String, password: String, access_token: UseStateHandle<String>) {
    spawn_local(async move {
        info!("Test");
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

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
