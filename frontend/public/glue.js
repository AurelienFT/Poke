const invoke = window.__TAURI__.invoke

export async function invokeLogin(username, password) {
    let captcha = document.getElementsByName('h-captcha-response')[0].value;
    return await invoke("login", {username: username, password: password, captcha: captcha});
}

export async function invokeGetCaptchaData() {
    let res = await invoke("get_captcha_data");
    return res;
}

export function test(data) {
    hcaptcha.setData('', {rqdata: data})
}