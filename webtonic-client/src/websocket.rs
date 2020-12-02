use bytes::Bytes;
use js_sys::{ArrayBuffer, Function, Promise, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, ErrorEvent, MessageEvent, WebSocket};
use webtonic_proto::WebTonicError;

use crate::console_log;

#[derive(Debug, Clone)]
pub(crate) struct WebSocketConnector {
    ws: WebSocket,
}

impl WebSocketConnector {
    pub(crate) async fn connect(uri: &str) -> Result<Self, WebTonicError> {
        let ws = WebSocket::new(uri).map_err(|_| WebTonicError::InvalidUrl)?;
        unset_message_handler(&ws);

        // NOTE: We can only process ArrayBuffers at the moment
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let connect_promise = Promise::new(&mut |resolve, error| {
            // Error callback
            let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                console_log(&format!("Error while connecting {:?}", e));
                error.call0(&JsValue::NULL).unwrap();
            }) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            // Connect callback
            let onopen_callback = Closure::wrap(Box::new(move |_| {
                console_log(&"Connected");
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

    pub(crate) async fn send(&self, request: &Bytes) -> Result<Bytes, WebTonicError> {
        let ws = self.ws.clone();

        let send_promise = Promise::new(&mut |resolve, error| {
            // Error callback
            let error_clone = error.clone();
            let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                console_log(&format!("Error while connecting {:?}", e));
                error.call0(&JsValue::NULL).unwrap();
            }) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            // Close callback
            let onclose_callback = Closure::wrap(Box::new(move |_| {
                console_log(&"Connection closed while waiting for a message");
                error_clone.call0(&JsValue::NULL).unwrap();
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            onclose_callback.forget();

            let ws_clone = ws.clone();
            // Message Callback
            let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                // We only want to receive one message, therefore we unset the message handler here
                unset_message_handler(&ws_clone);

                if let Ok(buf) = e.data().dyn_into::<ArrayBuffer>() {
                    resolve.call1(&JsValue::NULL, &JsValue::from(buf)).unwrap();
                } else {
                    console_log(&"Received unexpected message");
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();
        });

        // Send the actual data to the server
        match self.ws.send_with_u8_array(request) {
            Ok(_) => (),
            Err(e) => {
                console_log(&format!("Failed to send request {:?}", e));
                return Err(WebTonicError::ConnectionError);
            }
        }

        // Wait for message and parse it
        let buf = JsFuture::from(send_promise)
            .await
            .map_err(|_| WebTonicError::ConnectionError)?;
        let array = Uint8Array::new(&buf);
        let body = Bytes::from(array.to_vec());

        Ok(body)
    }
}

fn unset_message_handler(ws: &WebSocket) {
    // Message Callback
    let onmessage_callback = Closure::wrap(Box::new(move |_e: MessageEvent| {
        console_log(&"Received unexpected message");
    }) as Box<dyn FnMut(MessageEvent)>);
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    // TODO: Make this compliant with server once it is available
    #[wasm_bindgen_test]
    async fn websocket() {
        let ws = WebSocketConnector::connect("ws://localhost:1337")
            .await
            .unwrap();
        let msg = ws.send(&Bytes::from(b"WebTonic\n".to_vec())).await.unwrap();
        assert_eq!(&msg, &Bytes::from(b"Hello, WebTonic\n".to_vec()));
    }
}
