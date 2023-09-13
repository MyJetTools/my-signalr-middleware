use std::sync::Arc;

use my_http_server_web_sockets::MyWebSocket;

use crate::{MySignalrCallbacks, MySignalrConnection, SignalrConnectionsList};

pub async fn process_connect<
    TCtx: Send + Sync + Default + 'static,
    TMySignalrCallbacks: MySignalrCallbacks<TCtx = TCtx> + Send + Sync + 'static,
>(
    connections_callback: &Arc<TMySignalrCallbacks>,
    signal_r_list: &Arc<SignalrConnectionsList<TCtx>>,
    negotiation_version: usize,
    web_socket: Option<Arc<MyWebSocket>>,
) -> (Arc<MySignalrConnection<TCtx>>, String) {
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

    let signal_r_connection = MySignalrConnection::new(
        connection_id,
        conenction_token,
        negotiation_version,
        web_socket,
    );
    let signal_r_connection = Arc::new(signal_r_connection);

    connections_callback
        .connected(&signal_r_connection)
        .await
        .unwrap();

    signal_r_list
        .add_signalr_connection(signal_r_connection.clone())
        .await;

    (signal_r_connection, result)
}
