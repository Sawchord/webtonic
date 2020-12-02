use js_sys::{Function, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, ErrorEvent, MessageEvent, WebSocket};
use webtonic_proto::WebTonicError;

pub(crate) struct WebSocketConnector {
    ws: WebSocket,
}

impl WebSocketConnector {
    pub(crate) async fn connect(uri: String) -> Result<Self, WebTonicError> {
        let ws = WebSocket::new(&uri).map_err(|_| WebTonicError::InvalidUrl)?;

        // NOTE: We can only process ArrayBuffers at the moment
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let connect_promise = Promise::new(&mut |resolve, error| {
            // Error callback
            let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                console::log_1(&JsValue::from_str(&format!(
                    "Error while connecting {:?}",
                    e
                )));
                error.call0(&JsValue::NULL).unwrap();
            }) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            // Connect callback
            let onopen_callback = Closure::wrap(Box::new(move |_| {
                console::log_1(&JsValue::from_str(&"Connected"));
                resolve.call0(&JsValue::NULL).unwrap();
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();
        });

        JsFuture::from(connect_promise)
            .await
            .map_err(|_| WebTonicError::ConnectionError)?;
        Ok(Self { ws })
    }

    pub(crate) async fn send(&self) {
        todo!()
    }
}
