use bytes::Bytes;
use js_sys::{Promise, Uint8Array};
use std::sync::Arc;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver},
    Mutex,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use webtonic_proto::WebTonicError;

use crate::console_log;

#[derive(Debug, Clone)]
pub(crate) struct WebSocketConnector {
    ws: WebSocket,
    rx: Arc<Mutex<UnboundedReceiver<WsMessage>>>,
}

#[derive(Debug, Clone)]
enum WsMessage {
    Message(JsValue),
    Close,
    Error(JsValue),
}

impl WebSocketConnector {
    pub(crate) async fn connect(uri: &str) -> Result<Self, WebTonicError> {
        let ws = WebSocket::new(uri).map_err(|_| WebTonicError::InvalidUrl)?;
        let (tx, rx) = unbounded_channel::<WsMessage>();
        let rx = Arc::new(Mutex::new(rx));

        // NOTE: We can only process ArrayBuffers at the moment
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        let connect_promise = Promise::new(&mut |resolve, _| {
            let ws_clone = ws.clone();
            // Connect callback
            let onopen_callback = Closure::wrap(Box::new(move |_| {
                // Unset onopen
                ws_clone.set_onopen(None);

                resolve.call0(&JsValue::NULL).unwrap();
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();
        });

        // Error callback
        let tx_clone = tx.clone();
        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            tx_clone
                .send(WsMessage::Error(JsValue::from(e)))
                .expect("recevied error on closed socket");
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // Close callback
        let tx_clone = tx.clone();
        let onclose_callback = Closure::wrap(Box::new(move |_| {
            // TODO: Instead of send close just close the rx?
            tx_clone.send(WsMessage::Close).unwrap();
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        // Message Callback
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            tx.send(WsMessage::Message(JsValue::from(e.data())))
                .expect("received message on closed connection");
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        JsFuture::from(connect_promise)
            .await
            .map_err(|_| WebTonicError::ConnectionError)?;
        Ok(Self { ws, rx })
    }

    pub(crate) async fn send(&self, request: &Bytes) -> Result<Bytes, WebTonicError> {
        // Acquire rx, so we have exclusive access to the socket
        let mut guard = self.rx.lock().await;

        // Send the actual data to the server
        match self.ws.send_with_u8_array(request) {
            Ok(_) => (),
            Err(e) => {
                console_log(&format!("Failed to send request {:?}", e));
                return Err(WebTonicError::ConnectionError);
            }
        }

        // Now wait for the answer
        let buf = match guard.recv().await {
            Some(WsMessage::Message(msg)) => msg,
            Some(WsMessage::Error(e)) => {
                console_log(&format!("error while waiting for message {:?}", e));
                return Err(WebTonicError::ConnectionError);
            }
            None | Some(WsMessage::Close) => {
                return Err(WebTonicError::ConnectionClosed);
            }
        };

        // Parse answer
        let array = Uint8Array::new(&buf);
        let body = Bytes::from(array.to_vec());

        Ok(body)
    }
}

// Unset all message handler once the Connector gets dropped
impl Drop for WebSocketConnector {
    fn drop(&mut self) {
        self.ws.set_onclose(None);
        self.ws.set_onmessage(None);
        self.ws.set_onerror(None);
    }
}
