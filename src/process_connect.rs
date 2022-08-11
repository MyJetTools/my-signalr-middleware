use std::sync::Arc;

use my_http_server_web_sockets::MyWebSocket;

use crate::{signal_r_list::SignalrList, MySignalrCallbacks, MySignalrConnection};

pub async fn process_connect(
    connections_callback: &Arc<dyn MySignalrCallbacks + Send + Sync + 'static>,
    socket_io_list: &Arc<SignalrList>,
    web_socket: Option<Arc<MyWebSocket>>,
) -> (Arc<MySignalrConnection>, String) {
    let connection_id = uuid::Uuid::new_v4().to_string();

    let connection_id = connection_id.replace("-", "")[..8].to_string();

    let conenction_token = uuid::Uuid::new_v4().to_string();

    let result = crate::messages::generate_negotiate_response(
        connection_id.as_str(),
        conenction_token.as_str(),
    );

    let socket_io = MySignalrConnection::new(connection_id, conenction_token, web_socket);
    let socket_io_connection = Arc::new(socket_io);

    connections_callback
        .connected(socket_io_connection.clone())
        .await
        .unwrap();

    socket_io_list
        .add_socket_io(socket_io_connection.clone())
        .await;

    (socket_io_connection, result)
}
