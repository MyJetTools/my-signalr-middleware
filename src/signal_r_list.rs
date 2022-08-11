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

    pub async fn add_socket_io(&self, signalr_connection: Arc<MySignalrConnection>) {
        let web_socket = signalr_connection.get_web_socket().await;
        let mut write_access = self.sockets.write().await;
        write_access.sockets_by_connection_token.insert(
            signalr_connection.connection_token.clone(),
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
        web_socket_io: i64,
    ) -> Option<Arc<MySignalrConnection>> {
        let read_access = self.sockets.read().await;
        let result = read_access.sockets_by_web_socket_id.get(&web_socket_io)?;
        Some(result.clone())
    }

    pub async fn remove(&self, connection_token: &str) -> Option<Arc<MySignalrConnection>> {
        let removed_socket_io = {
            let mut write_access = self.sockets.write().await;
            write_access
                .sockets_by_connection_token
                .remove(connection_token)
        };

        if let Some(removed_socket_io) = &removed_socket_io {
            let web_socket = removed_socket_io.disconnect().await;
            if let Some(web_socket) = web_socket {
                let mut write_access = self.sockets.write().await;
                write_access.sockets_by_web_socket_id.remove(&web_socket.id);
            }
        } else {
            return None;
        }

        removed_socket_io
    }
}
