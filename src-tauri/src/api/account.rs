use crate::api::cookie_jar::CookieJar;
use reqwest::{Client,
    header::{ACCEPT, CONTENT_TYPE, COOKIE, HOST, PRAGMA, USER_AGENT},
};
use serde_json::json;
use serde::{Serialize, Deserialize};
use url::Url;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseSentCaptcha {
    pub error_id: i64,
    pub task_id: String,
    pub status: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolutionCaptcha {
    pub g_recaptcha_response: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseGetCaptchaResult {
    pub error_id: i64,
    pub task_id: String,
    pub status: String,
    pub solution: Option<SolutionCaptcha>,
}

pub enum Captcha {
    Resolved(String, CookieJar),
    ToResolveWithCapsolver,
}

pub struct Account {
    pub access_token: String,
    pub cookies: CookieJar,
}

impl Account {
    // Key Data Cookies
    pub async fn get_captcha_data() -> (String, String, CookieJar) {
        let body = json!({
            "acr_values": "",
            "claims": "",
            "client_id": "riot-client",
            "code_challenge": "",
            "code_challenge_method": "",
            "nonce": "dIIZ_afu0DfKKRdQc2KMLQ",
            "redirect_uri": "http://localhost/redirect",
            "response_type": "token id_token",
            "scope": "openid link ban lol_region account"
        }).to_string();
        let resp = Client::new()
            .post("https://auth.riotgames.com/api/v1/authorization")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(
                USER_AGENT,
                "RiotClient/70.0.0.247.1382 rso-auth (Windows;10;;Professional, x64)",
            )
            .header(PRAGMA, "no-cache")
            .body(body.clone())
            .send()
            .await
            .unwrap();
        let cookie = resp.headers().get("set-cookie").unwrap();
        let resp2 = Client::new()
            .post("https://auth.riotgames.com/api/v1/authorization")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(
                USER_AGENT,
                "RiotClient/70.0.0.247.1382 rso-auth (Windows;10;;Professional, x64)",
            )
            .header(PRAGMA, "no-cache")
            .header(COOKIE, cookie)
            .body(body)
            .send()
            .await
            .unwrap();

        let mut jar = CookieJar::new();
        for (key, value) in resp2.headers().iter() {
            if key == "set-cookie" {
                jar.add(value.to_str().unwrap().to_string());
            }
        }
        let json = json!({
        "apple": null,
        "campaign": null,
        "clientId": "riot-client",
        "code": null,
        "facebook": null,
        "gamecenter": null,
        "google": null,
        "language": "",
        "multifactor": null,
        "nintendo": null,
        "platform": "windows",
        "playstation": null,
        "remember": false,
        "riot_identity": {
            "campaign": null,
            "captcha": null,
            "language": "en_GB",
            "password": null,
            "remember": null,
            "state": "auth",
            "username": null
        },
        "riot_identity_signup": null,
        "rso": null,
        "sdkVersion": "23.8.0.1382",
        "type": "auth",
        "xbox": null
        }).to_string();
        let resp3 = Client::new()
            .post("https://authenticate.riotgames.com/api/v1/login")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(
                USER_AGENT,
                "RiotClient/70.0.0.247.1382 rso-authenticator (Windows;10;;Professional, x64)",
            )
            .header(COOKIE, jar.create_cookie_header())
            .body(json)
            .send()
            .await
            .unwrap();
        for (key, value) in resp3.headers().iter() {
            if key == "set-cookie" {
                jar.add(value.to_str().unwrap().to_string());
            }
        }
        let response = serde_json::from_str::<serde_json::Value>(&resp3.text().await.unwrap()).unwrap();
        let captcha = response
            .as_object()
            .unwrap()
            .get("captcha")
            .unwrap()
            .as_object()
            .unwrap()
            .get("hcaptcha")
            .unwrap()
            .as_object()
            .unwrap();
        (String::from(captcha.get("key").unwrap().as_str().unwrap()), String::from(captcha.get("data").unwrap().as_str().unwrap()), jar)
    }

    pub async fn login(username: String, password: String, captcha: Captcha) -> Account {
        let (solution, mut jar) = match captcha {
            Captcha::Resolved(answer, jar) => (answer, jar),
            Captcha::ToResolveWithCapsolver => {
                let cap_solver_api = std::env::var("CAPSOLVER_API_KEY").unwrap();
                let (website_key, rq_data, jar) = Account::get_captcha_data().await;
                let json = json!({
                  "clientKey": cap_solver_api,
                  "task": {
                    //Required. Can use HCaptchaTaskProxyless or HCaptchaTask
                    "type": "HCaptchaTaskProxyLess",
                    //Required
                    "websiteURL": "https://authenticate.riotgames.com/api/v1/login",
                    // Required
                    "websiteKey": website_key,
                    // Optional
                    "isInvisible": true,
                    // Optional
                    "enterprisePayload": {
                      //Optional, required if the site have HCaptcha Enterprise
                      "rqdata": rq_data
                    },
                    "userAgent": ""
                  }
                });
                let body = json.to_string();
                let resp4 = Client::new()
                    .post("https://api.capsolver.com/createTask")
                    .header(HOST, "api.capsolver.com")
                    .header(CONTENT_TYPE, "application/json")
                    .body(body)
                    .send()
                    .await
                    .unwrap();
                let text = resp4.text().await.unwrap();
                let response = serde_json::from_str::<ResponseSentCaptcha>(
                    &text,
                )
                .unwrap();
                let json = json!({
                    "clientKey": cap_solver_api,
                    "taskId": response.task_id,
                });
                let body = json.to_string();
                std::thread::sleep(std::time::Duration::from_secs(10));
                let resp5 = Client::new()
                    .post("https://api.capsolver.com/getTaskResult")
                    .header(HOST, "api.capsolver.com")
                    .header(CONTENT_TYPE, "application/json")
                    .body(body)
                    .send()
                    .await
                    .unwrap();
                let text = resp5.text().await.unwrap();
                println!("response5: {}", text);
                let response = serde_json::from_str::<ResponseGetCaptchaResult>(
                    &text,
                )
                .unwrap();
                (response
                    .solution
                    .unwrap()
                    .g_recaptcha_response, jar)
            }
        };
        let body = json!({
            "riot_identity": {
                "campaign": null,
                "captcha": format!("hcaptcha {}", solution),
                "language": "en_GB",
                "password": password,
                "remember": false,
                "state": null,
                "username": username
            },
            "type": "auth"
        }).to_string();
        println!("cookies: {}", jar.create_cookie_header());
        let resp6 = Client::new()
            .put("https://authenticate.riotgames.com/api/v1/login")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(
                USER_AGENT,
                "RiotClient/70.0.0.247.1382 rso-authenticator (Windows;10;;Professional, x64)",
            )
            .header(COOKIE, jar.create_cookie_header())
            .body(body)
            .send()
            .await
            .unwrap();
        let text = resp6.text().await.unwrap();
        println!("response6: {}", text);
        let response = serde_json::from_str::<serde_json::Value>(&text).unwrap();
        let token = response
            .as_object()
            .unwrap()
            .get("success")
            .unwrap()
            .as_object()
            .unwrap()
            .get("login_token")
            .unwrap()
            .as_str()
            .unwrap();
        let body = json!({
            "authentication_type": "RiotAuth",
            "code_verifier": "",
            "login_token": token,
            "persist_login": false
        }).to_string();
        let resp7 = Client::new()
            .post("https://auth.riotgames.com/api/v1/login-token")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(
                USER_AGENT,
                "RiotClient/70.0.0.247.1382 rso-auth (Windows;10;;Professional, x64)",
            )
            .header(COOKIE, jar.create_cookie_header())
            .body(body)
            .send()
            .await
            .unwrap();
        for (key, value) in resp7.headers().iter() {
            if key == "set-cookie" {
                jar.add(value.to_str().unwrap().to_string());
            }
        }
        let body = json!({
            "acr_values": "",
            "claims": "",
            "client_id": "riot-client",
            "code_challenge": "",
            "code_challenge_method": "",
            "nonce": "dIIZ_afu0DfKKRdQc2KMLQ",
            "redirect_uri": "http://localhost/redirect",
            "response_type": "token id_token",
            "scope": "openid link ban lol_region account"
        }).to_string();
        let resp8 = Client::new()
            .post("https://auth.riotgames.com/api/v1/authorization")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(
                USER_AGENT,
                "RiotClient/70.0.0.247.1382 rso-auth (Windows;10;;Professional, x64)",
            )
            .header(COOKIE, jar.create_cookie_header())
            .header(PRAGMA, "no-cache")
            .body(body)
            .send()
            .await
            .unwrap();
        let text = resp8.text().await.unwrap();
        let response = serde_json::from_str::<serde_json::Value>(&text).unwrap();
        let url = response
            .as_object()
            .unwrap()
            .get("response")
            .unwrap()
            .as_object()
            .unwrap()
            .get("parameters")
            .unwrap()
            .as_object()
            .unwrap()
            .get("uri")
            .unwrap()
            .as_str()
            .unwrap();
        let token_url = Url::parse(url).unwrap();
        let token = token_url
            .query_pairs()
            .find(|(key, _)| key == "access_token")
            .unwrap()
            .1
            .to_string();
        Account {
            access_token: token.to_string(),
            cookies: jar,
        }
    }
}
