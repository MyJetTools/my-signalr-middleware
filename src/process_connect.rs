use std::sync::Arc;

use my_http_server_web_sockets::MyWebSocket;

use crate::{signal_r_list::SignalrList, MySignalrCallbacks, MySignalrConnection};

pub async fn process_connect(
    connections_callback: &Arc<dyn MySignalrCallbacks + Send + Sync + 'static>,
    signalr_list: &Arc<SignalrList>,
    web_socket: Option<Arc<MyWebSocket>>,
) -> (Arc<MySignalrConnection>, String) {
    let connection_id = uuid::Uuid::new_v4().to_string();

    let connection_id = connection_id.replace("-", "")[..8].to_string();

    let conenction_token = uuid::Uuid::new_v4().to_string();

    let result = crate::messages::generate_negotiate_response(
        connection_id.as_str(),
        conenction_token.as_str(),
    );

    let signalr_connection = MySignalrConnection::new(connection_id, conenction_token, web_socket);
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
