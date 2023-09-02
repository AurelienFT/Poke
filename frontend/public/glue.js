const invoke = window.__TAURI__.invoke
console.log(window);

export async function invokeLogin(username, password) {
    let captcha = document.getElementsByName('h-captcha-response')[0].value;
    console.log(captcha);
    return await invoke("login", {username: username, password: password, captcha: captcha});
}

export async function invokeGetCaptchaData() {
    return await invoke("get_captcha_data");
}