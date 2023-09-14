use std::{
    net::SocketAddr,
    sync::{atomic::AtomicBool, Arc},
};

use hyper_tungstenite::tungstenite::Message;
use my_http_server_web_sockets::MyWebSocket;
use rust_extensions::{
    date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds},
    TaskCompletion,
};

use tokio::sync::Mutex;

#[cfg(feature = "with-ctx")]
use tokio::sync::RwLock;

use crate::SignalRParam;

pub struct MySignalrConnectionSingleThreaded {
    web_socket: Option<Arc<MyWebSocket>>,
    long_pooling: Option<TaskCompletion<String, String>>,
}

pub struct MySignalrConnection<TCtx: Send + Sync + 'static> {
    single_threaded: Mutex<MySignalrConnectionSingleThreaded>,
    pub connection_id: String,
    pub connection_token: Option<String>,
    pub created: DateTimeAsMicroseconds,
    pub last_incoming_moment: AtomicDateTimeAsMicroseconds,
    connected: AtomicBool,
    has_web_socket: AtomicBool,
    has_greeting: AtomicBool,
    pub negotiation_version: usize,
    pub ctx: TCtx,
}

impl<TCtx: Send + Sync + Default + 'static> MySignalrConnection<TCtx> {
    pub fn new(
        connection_id: String,
        connection_token: Option<String>,
        negotiation_version: usize,
        web_socket: Option<Arc<MyWebSocket>>,
    ) -> Self {
        let has_web_socket = web_socket.is_some();
        Self {
            single_threaded: Mutex::new(MySignalrConnectionSingleThreaded {
                web_socket,
                long_pooling: None,
            }),
            connection_id,
            connection_token,
            negotiation_version,
            created: DateTimeAsMicroseconds::now(),
            last_incoming_moment: AtomicDateTimeAsMicroseconds::now(),
            connected: AtomicBool::new(true),
            has_web_socket: AtomicBool::new(has_web_socket),
            has_greeting: AtomicBool::new(false),
            ctx: TCtx::default(),
        }
    }

    pub fn get_list_index(&self) -> &String {
        if let Some(token) = self.connection_token.as_ref() {
            token
        } else {
            &self.connection_id
        }
    }

    pub fn get_has_greeting(&self) -> bool {
        self.has_greeting.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_has_greeting(&self) {
        self.has_greeting
            .store(true, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn in_web_socket_model(&self) -> bool {
        self.has_web_socket
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn has_web_socket(&self, web_socket_id: i64) -> bool {
        let read_access = self.single_threaded.lock().await;
        if let Some(web_socket) = &read_access.web_socket {
            return web_socket.id == web_socket_id;
        }

        false
    }

    pub async fn get_web_socket(&self) -> Option<Arc<MyWebSocket>> {
        let read_access = self.single_threaded.lock().await;
        read_access.web_socket.clone()
    }

    pub fn update_incoming_activity(&self) {
        let now = DateTimeAsMicroseconds::now();
        self.last_incoming_moment.update(now);
    }

    pub fn get_last_incoming(&self) -> DateTimeAsMicroseconds {
        self.last_incoming_moment.as_date_time()
    }

    pub async fn send<'s>(&self, action_name: &str, parameter: &SignalRParam<'s>) {
        let web_socket = {
            let read_access = self.single_threaded.lock().await;
            read_access.web_socket.clone()
        };

        if let Some(web_socket) = web_socket {
            let mut result = Vec::new();

            result.extend_from_slice("{\"type\":1,\"target\":\"".as_bytes());
            result.extend_from_slice(action_name.as_bytes());
            result.extend_from_slice("\",\"arguments\":[".as_bytes());
            match parameter {
                SignalRParam::JsonObject(json_writer) => {
                    json_writer.build_into(&mut result);
                }
                SignalRParam::String(value) => {
                    let json_string = my_json::EscapedJsonString::new(value);
                    result.push(b'"');
                    result.extend_from_slice(json_string.as_str().as_bytes());
                    result.push(b'"');
                }
                SignalRParam::Number(number) => {
                    result.extend_from_slice(number.to_string().as_bytes());
                }
                SignalRParam::Float(value) => {
                    result.extend_from_slice(value.to_string().as_bytes());
                }
                SignalRParam::Boolean(value) => {
                    if *value {
                        result.extend_from_slice("true".as_bytes());
                    } else {
                        result.extend_from_slice("false".as_bytes());
                    }
                }
                SignalRParam::Raw(value) => {
                    for (index, item) in value.iter().enumerate() {
                        if index > 0 {
                            result.push(b',');
                        }
                        result.extend_from_slice(item.as_slice());
                    }
                }
                SignalRParam::None => {}
            }

            result.extend_from_slice("]}".as_bytes());
            result.push(30);

            web_socket
                .send_message(Message::Text(String::from_utf8(result).unwrap()))
                .await;
        }
    }

    pub async fn send_ping_payload(&self) {
        self.send_raw_payload(crate::messages::get_ping_payload().to_string())
            .await;
    }

    pub async fn send_raw_payload(&self, mut raw_payload: String) {
        let web_socket = {
            let read_access = self.single_threaded.lock().await;
            read_access.web_socket.clone()
        };

        raw_payload.push(30 as char);

        if let Some(web_socket) = web_socket {
            web_socket.send_message(Message::Text(raw_payload)).await;
        }
    }

    pub async fn add_web_socket(&self, web_socket: Arc<MyWebSocket>) {
        let new_id = web_socket.id;
        let mut write_access = self.single_threaded.lock().await;

        if let Some(old_websocket) = write_access.web_socket.replace(web_socket) {
            old_websocket
                .send_message(hyper_tungstenite::tungstenite::Message::Text(format!(
                    "Signalr WebSocket {} has been kicked by Websocket {} ",
                    old_websocket.id, new_id
                )))
                .await;
        }
    }

    pub async fn disconnect(&self) -> Option<Arc<MyWebSocket>> {
        let mut write_access = self.single_threaded.lock().await;

        self.connected
            .store(false, std::sync::atomic::Ordering::SeqCst);

        let mut result = None;

        if let Some(web_socket) = write_access.web_socket.take() {
            web_socket.disconnect().await;
            result = Some(web_socket);
        }

        if let Some(mut long_pooling) = write_access.long_pooling.take() {
            long_pooling.set_error(format!("Canceling this LongPool since we disconnect it."));
        }

        result
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub async fn get_addr(&self) -> Option<SocketAddr> {
        let read_access = self.single_threaded.lock().await;
        if let Some(web_socket) = &read_access.web_socket {
            return Some(web_socket.addr);
        }
        None
    }
}
