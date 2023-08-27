use log::info;
use wasm_bindgen::{JsValue, prelude::*};
use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

#[wasm_bindgen(module = "/public/glue.js")]
extern "C" {
    #[wasm_bindgen(js_name = invokeHello, catch)]
    pub async fn hello(name: String) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = invokeLogin, catch)]
    pub async fn login(username: String, password: String) -> Result<JsValue, JsValue>;
}

#[function_component]
fn App() -> Html {
    let username = use_state_eq(|| "".to_string());
    let password = use_state_eq(|| "".to_string());
    let access_token = use_state(|| "".to_string());

    // Execute tauri command via effects.
    // The effect will run every time `name` changes.
    {
        let access_token = access_token.clone();
        use_effect_with_deps(
            move |(username, password)| {
                perform_login(username.clone(), password.clone(), access_token);
                || ()
            },
            ((*username).clone(), (*password).clone()),
        );
    }

    let message = (*access_token).clone();
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    html! {
        <div>
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
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
