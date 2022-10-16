use std::{collections::HashMap, sync::Arc};

use my_http_server::HttpFailResult;

use crate::MySignalrConnection;

#[async_trait::async_trait]
pub trait MySignalrCallbacks {
    type TCtx: Send + Sync + Default + 'static;
    async fn connected(
        &self,
        connection: &Arc<MySignalrConnection<Self::TCtx>>,
    ) -> Result<(), HttpFailResult>;
    async fn disconnected(&self, connection: &Arc<MySignalrConnection<Self::TCtx>>);
    async fn on_ping(&self, connection: &Arc<MySignalrConnection<Self::TCtx>>);
    async fn on(
        &self,
        connection: &Arc<MySignalrConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        action_name: &str,
        data: &[u8],
    );
}

#[async_trait::async_trait]
pub trait MySignalrTransportCallbacks {
    type TCtx: Send + Sync + Default + 'static;
    async fn connected(
        &self,
        connection: &Arc<MySignalrConnection<Self::TCtx>>,
    ) -> Result<(), HttpFailResult>;
    async fn disconnected(&self, connection: &Arc<MySignalrConnection<Self::TCtx>>);
    async fn on_ping(&self, connection: &Arc<MySignalrConnection<Self::TCtx>>);
}

#[async_trait::async_trait]
pub trait MySignalrPayloadCallbacks {
    type TCtx: Send + Sync + Default + 'static;
    async fn on(
        &self,
        signalr_connection: &Arc<MySignalrConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        action_name: &str,
        data: &[u8],
    );
}
