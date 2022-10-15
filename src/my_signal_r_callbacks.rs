use std::{collections::HashMap, sync::Arc};

use my_http_server::HttpFailResult;

use crate::MySignalrConnection;

#[async_trait::async_trait]
pub trait MySignalrCallbacks {
    type TCtx: Send + Sync + 'static;
    async fn connected(
        &self,
        signalr_connection: &Arc<MySignalrConnection<Self::TCtx>>,
    ) -> Result<(), HttpFailResult>;
    async fn disconnected(&self, signalr_connection: &Arc<MySignalrConnection<Self::TCtx>>);
    async fn on(
        &self,
        signalr_connection: &Arc<MySignalrConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        action_name: &str,
        data: &[u8],
    );
}
