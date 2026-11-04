//! Adapter for Tauri window management and Tauri IPC commands.

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Invokes a zero-argument Tauri command on the main window.
pub fn invoke_tauri_window_command(cmd: &'static str) {
    if let Some(window) = web_sys::window()
        && let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        && let Ok(core_obj) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("core"))
        && let Ok(invoke_fn) = js_sys::Reflect::get(&core_obj, &JsValue::from_str("invoke"))
        && let Ok(func) = invoke_fn.dyn_into::<js_sys::Function>()
    {
        let _ = func.call1(&core_obj, &JsValue::from_str(cmd));
    }
}

/// Invokes Tauri `send_device_config` to transmit UDP OTA Wi-Fi configuration.
pub async fn invoke_send_device_config(
    target_ip: String,
    serial_number: String,
    device_id: String,
    wifi_ssid: String,
    wifi_password: String,
) -> Result<String, String> {
    if let Some(window) = web_sys::window()
        && let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        && let Ok(core_obj) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("core"))
        && let Ok(invoke_fn) = js_sys::Reflect::get(&core_obj, &JsValue::from_str("invoke"))
        && let Ok(func) = invoke_fn.dyn_into::<js_sys::Function>()
    {
        let args = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &args,
            &JsValue::from_str("targetIp"),
            &JsValue::from_str(&target_ip),
        );
        let _ = js_sys::Reflect::set(
            &args,
            &JsValue::from_str("serialNumber"),
            &JsValue::from_str(&serial_number),
        );
        let _ = js_sys::Reflect::set(
            &args,
            &JsValue::from_str("deviceId"),
            &JsValue::from_str(&device_id),
        );
        let _ = js_sys::Reflect::set(
            &args,
            &JsValue::from_str("wifiSsid"),
            &JsValue::from_str(&wifi_ssid),
        );
        let _ = js_sys::Reflect::set(
            &args,
            &JsValue::from_str("wifiPassword"),
            &JsValue::from_str(&wifi_password),
        );

        let promise = match func.call2(&core_obj, &JsValue::from_str("send_device_config"), &args) {
            Ok(val) => val.dyn_into::<js_sys::Promise>().ok(),
            Err(e) => return Err(format!("Invoke failed: {:?}", e)),
        };

        if let Some(p) = promise {
            let fut = wasm_bindgen_futures::JsFuture::from(p);
            match fut.await {
                Ok(res) => Ok(res.as_string().unwrap_or_else(|| "Sukces!".to_string())),
                Err(err) => Err(err
                    .as_string()
                    .unwrap_or_else(|| "Błąd połączenia UDP z urządzeniem".to_string())),
            }
        } else {
            Err("Failed to acquire Promise".to_string())
        }
    } else {
        Err("Tauri environment not available".to_string())
    }
}
