use std::{sync::Arc, time::Duration};

use hyper_tungstenite::tungstenite::Message;
use my_http_server::HttpFailResult;
use my_http_server_web_sockets::{MyWebSocket, WebSocketMessage};
use my_json::json_reader::JsonFirstLineReader;
#[cfg(feature = "my-telemetry")]
use my_telemetry::MyTelemetryContext;
#[cfg(feature = "my-telemetry")]
use my_telemetry::TelemetryEventTagsBuilder;

use crate::{
    messages::SignalrMessage, MySignalrCallbacks, MySignalrConnection, SignalrConnectionsList,
};

pub struct WebSocketCallbacks<TCtx: Send + Sync + Default + 'static> {
    pub signalr_list: Arc<SignalrConnectionsList<TCtx>>,
    pub my_signal_r_callbacks: Arc<dyn MySignalrCallbacks<TCtx = TCtx> + Send + Sync + 'static>,
}

#[async_trait::async_trait]
impl<TCtx: Send + Sync + Default + 'static> my_http_server_web_sockets::MyWebSocketCallback
    for WebSocketCallbacks<TCtx>
{
    async fn connected(
        &self,
        my_web_socket: Arc<MyWebSocket>,
        disconnect_timeout: Duration,
    ) -> Result<(), HttpFailResult> {
        #[cfg(feature = "debug_ws")]
        println!("connected web_socket:{}", my_web_socket.id);

        if let Some(query_string) = my_web_socket.get_query_string() {
            let connection_token = query_string.get_optional("id");

            if connection_token.is_none() {
                my_web_socket
                    .send_message(Message::Text("id query parameter is missing".to_string()))
                    .await;
                return Ok(());
            }

            let connection_token = connection_token.unwrap();

            match self
                .signalr_list
                .assign_web_socket(connection_token.value, my_web_socket.clone())
                .await
            {
                Some(signalr_connection) => {
                    tokio::spawn(super::signalr_liveness_loop::start(
                        self.my_signal_r_callbacks.clone(),
                        self.signalr_list.clone(),
                        signalr_connection,
                        disconnect_timeout,
                    ));
                }
                None => {
                    my_web_socket
                        .send_message(Message::Text(format!(
                            "SignalR with connection_token {} is not found",
                            connection_token.value,
                        )))
                        .await;

                    return Ok(());
                }
            };
        }

        Ok(())
    }

    async fn disconnected(&self, my_web_socket: Arc<MyWebSocket>) {
        #[cfg(feature = "debug_ws")]
        println!("disconnected web_socket:{}", my_web_socket.id);
        let find_result = self
            .signalr_list
            .get_by_web_socket_id(my_web_socket.id)
            .await;

        if let Some(signalr_connection) = find_result {
            crate::process_disconnect(
                &self.signalr_list,
                &signalr_connection,
                &self.my_signal_r_callbacks,
            )
            .await;
        }
    }
    async fn on_message(&self, my_web_socket: Arc<MyWebSocket>, message: WebSocketMessage) {
        #[cfg(feature = "debug_ws")]
        println!("Websocket{}, MSG: {:?}", my_web_socket.id, message);

        let signal_r = self
            .signalr_list
            .get_by_web_socket_id(my_web_socket.id)
            .await;

        if let Some(signalr_connection) = signal_r.as_ref() {
            signalr_connection.update_incoming_activity();

            if let WebSocketMessage::String(value) = &message {
                if signalr_connection.get_has_greeting() {
                    let packet_type = get_payload_type(value);

                    if packet_type == "1" {
                        #[cfg(feature = "my-telemetry")]
                        let ctx = MyTelemetryContext::new();

                        #[cfg(feature = "my-telemetry")]
                        let started = rust_extensions::date_time::DateTimeAsMicroseconds::now();

                        let message = SignalrMessage::parse(value);

                        #[cfg(feature = "my-telemetry")]
                        let ctx_spawned = ctx.clone();

                        let signal_r_callbacks = self.my_signal_r_callbacks.clone();

                        let connection_spawned = signalr_connection.clone();

                        let target = message.target.to_string();

                        let arguments = message.arguments.to_vec();

                        let _result = tokio::spawn(async move {
                            #[cfg(feature = "my-telemetry")]
                            let mut signal_r_telemetry = crate::SignalRTelemetry::new(ctx_spawned);
                            signal_r_callbacks
                                .on(
                                    connection_spawned,
                                    message.headers,
                                    target,
                                    arguments,
                                    #[cfg(feature = "my-telemetry")]
                                    &mut signal_r_telemetry,
                                )
                                .await;
                            #[cfg(feature = "my-telemetry")]
                            signal_r_telemetry.tags
                        })
                        .await;

                        #[cfg(feature = "my-telemetry")]
                        match _result {
                            Ok(tags) => {
                                my_telemetry::TELEMETRY_INTERFACE
                                    .write_success(
                                        &ctx,
                                        started,
                                        message.target.to_string(),
                                        format!("Executed Ok",),
                                        tags.add_ip(my_web_socket.addr.ip().to_string()).build(),
                                    )
                                    .await;
                            }
                            Err(err) => {
                                my_telemetry::TELEMETRY_INTERFACE
                                    .write_fail(
                                        &ctx,
                                        started,
                                        message.target.to_string(),
                                        format!("{:?}", err),
                                        TelemetryEventTagsBuilder::new()
                                            .add_ip(my_web_socket.addr.ip().to_string())
                                            .build(),
                                    )
                                    .await;
                            }
                        }
                    }

                    if packet_type == "6" {
                        signalr_connection.send_ping_payload().await;
                    }
                } else {
                    read_first_payload(signalr_connection, value).await
                }
            }
        }
    }
}

fn get_payload_type(payload: &str) -> &str {
    let json_reader = JsonFirstLineReader::new(payload.as_bytes());
    for line in json_reader {
        let line = line.unwrap();
        if line.get_name().unwrap() == "type" {
            let result = line.get_value().unwrap();
            return result.as_str().unwrap();
        }
    }

    panic!("Packet type is not found")
}

async fn read_first_payload<TCtx: Send + Sync + Default + 'static>(
    signalr_connection: &Arc<MySignalrConnection<TCtx>>,
    payload: &str,
) {
    let json_reader = JsonFirstLineReader::new(payload.as_bytes());

    let mut protocol = false;
    let mut version = false;

    for line in json_reader {
        let line = line.unwrap();

        if line.get_name().unwrap() == "protocol" {
            protocol = true;
        }
        if line.get_name().unwrap() == "version" {
            version = true;
        }
    }

    if protocol == true && version == true {
        signalr_connection.set_has_greeting();
        signalr_connection.send_raw_payload("{}".to_string()).await;
    }
}
