use std::{collections::HashMap, sync::Arc};

use my_http_server_web_sockets::MyWebSocket;
use tokio::sync::RwLock;

use crate::MySignalrConnection;

struct SignalrListInner {
    sockets_by_web_socket_id: HashMap<i64, Arc<MySignalrConnection>>,
    sockets_by_connection_token: HashMap<String, Arc<MySignalrConnection>>,
}

pub struct SignalrList {
    sockets: RwLock<SignalrListInner>,
}

impl SignalrList {
    pub fn new() -> Self {
        Self {
            sockets: RwLock::new(SignalrListInner {
                sockets_by_web_socket_id: HashMap::new(),
                sockets_by_connection_token: HashMap::new(),
            }),
        }
    }

    pub async fn add_signalr_connection(&self, signalr_connection: Arc<MySignalrConnection>) {
        let web_socket = signalr_connection.get_web_socket().await;
        let mut write_access = self.sockets.write().await;
        write_access.sockets_by_connection_token.insert(
            signalr_connection.get_list_index().to_string(),
            signalr_connection.clone(),
        );

        if let Some(web_socket) = web_socket {
            write_access
                .sockets_by_web_socket_id
                .insert(web_socket.id, signalr_connection);
        }
    }

    pub async fn assign_web_socket(
        &self,
        connection_token: &str,
        web_socket: Arc<MyWebSocket>,
    ) -> Option<Arc<MySignalrConnection>> {
        let found = {
            let mut write_access = self.sockets.write().await;

            let found = {
                if let Some(found) = write_access
                    .sockets_by_connection_token
                    .get(connection_token)
                {
                    Some(found.clone())
                } else {
                    None
                }
            };

            if let Some(found) = found {
                write_access
                    .sockets_by_web_socket_id
                    .insert(web_socket.id, found.clone());
                Some(found)
            } else {
                None
            }
        };

        if let Some(found) = found {
            found.add_web_socket(web_socket).await;
            Some(found)
        } else {
            None
        }
    }

    pub async fn get_by_connection_token(
        &self,
        connection_token: &str,
    ) -> Option<Arc<MySignalrConnection>> {
        let read_access = self.sockets.read().await;
        let result = read_access
            .sockets_by_connection_token
            .get(connection_token)?;
        Some(result.clone())
    }

    pub async fn get_by_web_socket_id(
        &self,
        web_socket_id: i64,
    ) -> Option<Arc<MySignalrConnection>> {
        let read_access = self.sockets.read().await;
        let result = read_access.sockets_by_web_socket_id.get(&web_socket_id)?;
        Some(result.clone())
    }

    pub async fn remove(&self, connection_token: &str) -> Option<Arc<MySignalrConnection>> {
        let removed_signalr_connection = {
            let mut write_access = self.sockets.write().await;
            write_access
                .sockets_by_connection_token
                .remove(connection_token)
        };

        if let Some(removed_signalr_connection) = &removed_signalr_connection {
            let web_socket = removed_signalr_connection.disconnect().await;
            if let Some(web_socket) = web_socket {
                let mut write_access = self.sockets.write().await;
                write_access.sockets_by_web_socket_id.remove(&web_socket.id);
            }
        } else {
            return None;
        }

        removed_signalr_connection
    }
}
