use log::info;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::prelude::*;

use yew_hcaptcha::HCaptcha;

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

    if (*captcha_data).is_none() {
        return html! {
            <div>
                <p>{ "Loading captcha..." }</p>
            </div>
        }
    }

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

    let on_load_captcha = Callback::from(move |_: ()| {
        if let Some(data) = &*captcha_data {
            test(data.data.clone());
        } else {
            ()
        }
    });

    html! {
        <div>
            <input
                type="text"
                oninput={on_input_username}
            />
            <input
                type="password"
                oninput={on_input_password}
            />
            <button onclick={on_login}>{ "Connect" }</button>
            <HCaptcha site_key={"019f1553-3845-481c-a6f5-5a60ccf6d830"} on_load={on_load_captcha}/>
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


fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
