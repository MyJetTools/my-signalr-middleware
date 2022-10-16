use std::{collections::HashMap, sync::Arc};

use my_http_server::HttpFailResult;

use crate::{
    MySignalrCallbacks, MySignalrConnection, MySignalrPayloadCallbacks, MySignalrTransportCallbacks,
};

pub struct MySignalrActions<TCtx: Send + Sync + Default + 'static> {
    pub transport_callbacks:
        Option<Arc<dyn MySignalrTransportCallbacks<TCtx = TCtx> + Send + Sync + 'static>>,
    actions:
        HashMap<String, Arc<dyn MySignalrPayloadCallbacks<TCtx = TCtx> + Send + Sync + 'static>>,
}

impl<TCtx: Send + Sync + Default + 'static> MySignalrActions<TCtx> {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            transport_callbacks: None,
        }
    }

    pub fn add_action(
        &mut self,
        action: String,
        callback: Arc<dyn MySignalrPayloadCallbacks<TCtx = TCtx> + Send + Sync + 'static>,
    ) {
        if self.actions.contains_key(&action) {
            panic!("Signalr action already registered: {}", action);
        }

        self.actions.insert(action, callback);
    }
}

#[async_trait::async_trait]
impl<TCtx: Send + Sync + Default + 'static> MySignalrCallbacks for MySignalrActions<TCtx> {
    type TCtx = TCtx;

    async fn connected(
        &self,
        connection: &Arc<MySignalrConnection<Self::TCtx>>,
    ) -> Result<(), HttpFailResult> {
        if let Some(c) = self.transport_callbacks.as_ref() {
            c.connected(connection).await
        } else {
            Ok(())
        }
    }
    async fn disconnected(&self, connection: &Arc<MySignalrConnection<Self::TCtx>>) {
        if let Some(c) = self.transport_callbacks.as_ref() {
            c.disconnected(connection).await
        }
    }

    async fn on_ping(&self, connection: &Arc<MySignalrConnection<Self::TCtx>>) {
        if let Some(c) = self.transport_callbacks.as_ref() {
            c.on_ping(connection).await
        }
    }
    async fn on(
        &self,
        signalr_connection: &Arc<MySignalrConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        action_name: &str,
        data: &[u8],
    ) {
        if let Some(action) = self.actions.get(action_name) {
            action
                .on(signalr_connection, headers, action_name, data)
                .await;
        }
    }
}
