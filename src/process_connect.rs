use std::sync::Arc;

use my_http_server_web_sockets::MyWebSocket;

use crate::{signal_r_list::SignalrList, MySignalrCallbacks, MySignalrConnection};

pub async fn process_connect(
    connections_callback: &Arc<dyn MySignalrCallbacks + Send + Sync + 'static>,
    signalr_list: &Arc<SignalrList>,
    negotiation_version: usize,
    web_socket: Option<Arc<MyWebSocket>>,
) -> (Arc<MySignalrConnection>, String) {
    let mut connection_id = uuid::Uuid::new_v4().to_string();
    connection_id = connection_id.replace("-", "");

    let conenction_token = if negotiation_version == 0 {
        None
    } else {
        let mut connection_token = uuid::Uuid::new_v4().to_string();
        connection_token = connection_token.replace("-", "");
        Some(connection_token)
    };

    let result = crate::messages::generate_negotiate_response(
        negotiation_version,
        connection_id.as_str(),
        &conenction_token,
    );

    let signalr_connection = MySignalrConnection::new(
        connection_id,
        conenction_token,
        negotiation_version,
        web_socket,
    );
    let signalr_connection = Arc::new(signalr_connection);

    connections_callback
        .connected(signalr_connection.clone())
        .await
        .unwrap();

    signalr_list
        .add_signalr_connection(signalr_connection.clone())
        .await;

    (signalr_connection, result)
}
