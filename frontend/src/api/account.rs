use crate::api::cookie_jar::CookieJar;
use reqwest::{
    blocking::Client,
    header::{ACCEPT, CONTENT_TYPE, COOKIE, HOST, PRAGMA, USER_AGENT},
};
use serde_json::json;

pub struct Account {
    pub access_token: String,
    pub cookies: CookieJar,
}

impl Account {
    pub fn login(username: String, password: String) -> Account {
        let json = json!({
            "acr_values": "",
            "claims": "",
            "client_id": "riot-client",
            "code_challenge": "",
            "code_challenge_method": "",
            "nonce": "dIIZ_afu0DfKKRdQc2KMLQ",
            "redirect_uri": "http://localhost/redirect",
            "response_type": "token id_token",
            "scope": "openid link ban lol_region account"
        });
        let body = json.to_string();
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
            .unwrap();

        let mut jar = CookieJar::new();
        for (key, value) in resp2.headers().iter() {
            if key == "set-cookie" {
                jar.add(value.to_str().unwrap().to_string());
            }
        }
        let json = r#"{
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
        }"#;
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
            .unwrap();
        for (key, value) in resp3.headers().iter() {
            if key == "set-cookie" {
                jar.add(value.to_str().unwrap().to_string());
            }
        }
        let response = serde_json::from_str::<serde_json::Value>(&resp3.text().unwrap()).unwrap();
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
        let json = json!({
          "clientKey": "CAP-A72E3B2EC4262C281D2D8DEEFF2F4C2A",
          "task": {
            //Required. Can use HCaptchaTaskProxyless or HCaptchaTask
            "type": "HCaptchaTaskProxyLess",
            //Required
            "websiteURL": "https://authenticate.riotgames.com/api/v1/login",
            // Required
            "websiteKey": captcha.get("key").unwrap().as_str().unwrap(),
            // Optional
            "isInvisible": true,
            // Optional
            "enterprisePayload": {
              //Optional, required if the site have HCaptcha Enterprise
              "rqdata": captcha.get("data").unwrap().as_str().unwrap()
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
            .unwrap();
        let response = serde_json::from_str::<serde_json::value::Map<String, serde_json::Value>>(
            &resp4.text().unwrap(),
        )
        .unwrap();
        let json = json!({
        "clientKey": "CAP-A72E3B2EC4262C281D2D8DEEFF2F4C2A",
        "taskId": response.get("taskId").unwrap().as_str().unwrap(),
        });
        let body = json.to_string();
        std::thread::sleep(std::time::Duration::from_secs(10));
        let resp5 = Client::new()
            .post("https://api.capsolver.com/getTaskResult")
            .header(HOST, "api.capsolver.com")
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .unwrap();
        let response = serde_json::from_str::<serde_json::value::Map<String, serde_json::Value>>(
            &resp5.text().unwrap(),
        )
        .unwrap();
        let solution = response
            .get("solution")
            .unwrap()
            .as_object()
            .unwrap()
            .get("gRecaptchaResponse")
            .unwrap()
            .as_str()
            .unwrap();
        let json = json!({
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
        });
        let body = json.to_string();
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
            .unwrap();
        let text = resp6.text().unwrap();
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
        let json = json!({
            "authentication_type": "RiotAuth",
            "code_verifier": "",
            "login_token": token,
            "persist_login": false
        });
        let body = json.to_string();
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
            .unwrap();
        for (key, value) in resp7.headers().iter() {
            if key == "set-cookie" {
                jar.add(value.to_str().unwrap().to_string());
            }
        }
        let json = json!({
            "acr_values": "",
            "claims": "",
            "client_id": "riot-client",
            "code_challenge": "",
            "code_challenge_method": "",
            "nonce": "dIIZ_afu0DfKKRdQc2KMLQ",
            "redirect_uri": "http://localhost/redirect",
            "response_type": "token id_token",
            "scope": "openid link ban lol_region account"
        });
        let body = json.to_string();
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
            .unwrap();
        let text = resp8.text().unwrap();
        let response = serde_json::from_str::<serde_json::Value>(&text).unwrap();
        let token = response
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
        Account {
            access_token: token.to_string(),
            cookies: jar,
        }
    }
}
