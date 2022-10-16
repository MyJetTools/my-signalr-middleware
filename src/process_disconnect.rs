use std::sync::Arc;

use crate::{MySignalrCallbacks, MySignalrConnection, SignalrConnectionsList};

pub async fn process_disconnect<TCtx: Send + Sync + Default + 'static>(
    sockets_list: &Arc<SignalrConnectionsList<TCtx>>,
    signalr_connection: &Arc<MySignalrConnection<TCtx>>,
    connect_events: &Arc<dyn MySignalrCallbacks<TCtx = TCtx> + Send + Sync + 'static>,
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
        connect_events.disconnected(&removed_connection).await;
    }
}
