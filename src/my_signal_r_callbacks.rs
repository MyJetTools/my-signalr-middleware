use std::sync::Arc;

use my_http_server::HttpFailResult;

use crate::MySignalrConnection;

#[async_trait::async_trait]
pub trait MySignalrCallbacks {
    async fn connected(&self, socket_io: Arc<MySignalrConnection>) -> Result<(), HttpFailResult>;
    async fn disconnected(&self, socket_io: Arc<MySignalrConnection>);
    async fn on(&self, action_name: &str, data: &str);
}
