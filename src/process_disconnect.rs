use std::sync::Arc;

use crate::{signal_r_list::SignalrList, MySignalrCallbacks, MySignalrConnection};

pub async fn process_disconnect(
    sockets_list: &Arc<SignalrList>,
    signalr_connection: &Arc<MySignalrConnection>,
    connect_events: &Arc<dyn MySignalrCallbacks + Send + Sync + 'static>,
) {
    let removed_connection = sockets_list
        .remove(signalr_connection.get_list_index())
        .await;

    if let Some(removed_connection) = removed_connection {
        #[cfg(feature = "debug_ws")]
        println!(
            "Signalr {} is diconnectd with connection token {:?}",
            removed_connection.connection_id, removed_connection.connection_token
        );
        connect_events.disconnected(removed_connection).await;
    }
}
