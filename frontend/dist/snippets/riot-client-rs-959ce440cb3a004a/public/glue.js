const invoke = window.__TAURI__.invoke
console.log(window);

export async function invokeHello(name) {
    return await invoke("hello", {name: name});
}