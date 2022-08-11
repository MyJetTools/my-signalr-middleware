use std::{collections::HashMap, sync::Arc};

use my_http_server::HttpFailResult;

use crate::MySignalrConnection;

#[async_trait::async_trait]
pub trait MySignalrCallbacks {
    async fn connected(
        &self,
        signalr_connection: &Arc<MySignalrConnection>,
    ) -> Result<(), HttpFailResult>;
    async fn disconnected(&self, signalr_connection: &Arc<MySignalrConnection>);
    async fn on(
        &self,
        signalr_connection: &Arc<MySignalrConnection>,
        headers: Option<HashMap<String, String>>,
        action_name: &str,
        data: &[u8],
    );
}
