use std::{collections::HashMap, sync::Arc};

use my_http_server_web_sockets::MyWebSocket;
use rust_extensions::lazy::LazyVec;
use tokio::sync::RwLock;

use crate::MySignalrConnection;

struct SignalrListInner<TCtx: Send + Sync + 'static> {
    sockets_by_web_socket_id: HashMap<i64, Arc<MySignalrConnection<TCtx>>>,
    sockets_by_connection_token: HashMap<String, Arc<MySignalrConnection<TCtx>>>,
    tags: crate::Tags,
}

pub struct SignalrConnectionsList<TCtx: Send + Sync + Default + 'static> {
    sockets: RwLock<SignalrListInner<TCtx>>,
}

impl<TCtx: Send + Sync + Default + 'static> SignalrConnectionsList<TCtx> {
    pub fn new() -> Self {
        Self {
            sockets: RwLock::new(SignalrListInner {
                sockets_by_web_socket_id: HashMap::new(),
                sockets_by_connection_token: HashMap::new(),
                tags: crate::Tags::new(),
            }),
        }
    }

    pub async fn add_signalr_connection(&self, signalr_connection: Arc<MySignalrConnection<TCtx>>) {
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
    ) -> Option<Arc<MySignalrConnection<TCtx>>> {
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
    ) -> Option<Arc<MySignalrConnection<TCtx>>> {
        let read_access = self.sockets.read().await;
        let result = read_access
            .sockets_by_connection_token
            .get(connection_token)?;
        Some(result.clone())
    }

    pub async fn get_by_web_socket_id(
        &self,
        web_socket_id: i64,
    ) -> Option<Arc<MySignalrConnection<TCtx>>> {
        let read_access = self.sockets.read().await;
        let result = read_access.sockets_by_web_socket_id.get(&web_socket_id)?;
        Some(result.clone())
    }

    pub async fn get_all(&self) -> Option<Vec<Arc<MySignalrConnection<TCtx>>>> {
        let read_access = self.sockets.read().await;

        if read_access.sockets_by_connection_token.is_empty() {
            return None;
        }

        let result = read_access
            .sockets_by_connection_token
            .values()
            .map(|v| v.clone())
            .collect();

        Some(result)
    }

    pub async fn find_first<TFn: Fn(&MySignalrConnection<TCtx>) -> bool>(
        &self,
        filter: TFn,
    ) -> Option<Arc<MySignalrConnection<TCtx>>> {
        let read_access = self.sockets.read().await;

        for connection in read_access.sockets_by_connection_token.values() {
            if filter(connection) {
                return Some(connection.clone());
            }
        }

        None
    }

    pub async fn filter<TFn: Fn(&MySignalrConnection<TCtx>) -> bool>(
        &self,
        filter: TFn,
    ) -> Option<Vec<Arc<MySignalrConnection<TCtx>>>> {
        let read_access = self.sockets.read().await;
        let mut result = LazyVec::new();

        for connection in read_access.sockets_by_connection_token.values() {
            if filter(connection) {
                result.add(connection.clone());
            }
        }

        result.result
    }

    pub async fn remove(&self, connection_token: &str) -> Option<Arc<MySignalrConnection<TCtx>>> {
        let removed_signalr_connection = {
            let mut write_access = self.sockets.write().await;
            let removed = write_access
                .sockets_by_connection_token
                .remove(connection_token);

            if let Some(removed) = &removed {
                write_access.tags.remove_connection(&removed.connection_id);
            }

            removed
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

    pub async fn add_tag_to_connection(
        &self,
        ctx: &MySignalrConnection<TCtx>,
        key: &str,
        value: &str,
    ) {
        let mut write_access = self.sockets.write().await;

        if write_access
            .sockets_by_connection_token
            .contains_key(ctx.get_list_index())
        {
            write_access.tags.add_tag(&ctx.get_list_index(), key, value);
        }
    }

    pub async fn remove_tag_from_connection(
        &self,
        ctx: Arc<MySignalrConnection<TCtx>>,
        key: &str,
        value: &str,
    ) {
        let mut write_access = self.sockets.write().await;

        if write_access
            .sockets_by_connection_token
            .contains_key(ctx.get_list_index())
        {
            write_access
                .tags
                .remove_tag(&ctx.get_list_index(), key, value);
        }
    }

    pub async fn get_tagged_connections_with_value(
        &self,
        key: &str,
        value: &str,
    ) -> Option<Vec<Arc<MySignalrConnection<TCtx>>>> {
        let read_access = self.sockets.read().await;

        if let Some(id_s) = read_access
            .tags
            .get_tagged_connections_with_value(key, value)
        {
            let mut result = Vec::with_capacity(id_s.len());

            for id in &id_s {
                if let Some(connection) = read_access.sockets_by_connection_token.get(id) {
                    result.push(connection.clone());
                }
            }

            return Some(result);
        }

        None
    }

    pub async fn get_tagged_connections(
        &self,
        key: &str,
    ) -> Option<Vec<Arc<MySignalrConnection<TCtx>>>> {
        let read_access = self.sockets.read().await;

        if let Some(id_s) = read_access.tags.get_tagged_connections(key) {
            let mut result = Vec::with_capacity(id_s.len());

            for id in &id_s {
                if let Some(connection) = read_access.sockets_by_connection_token.get(id) {
                    result.push(connection.clone());
                }
            }

            return Some(result);
        }

        None
    }
}
