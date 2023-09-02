const invoke = window.__TAURI__.invoke
console.log(window);

export async function invokeLogin(username, password, callback) {
    return await invoke("login", {username: username, password: password, callback: callback});
}

export async function invokeGetCaptchaData() {
    return await invoke("get_captcha_data");
}