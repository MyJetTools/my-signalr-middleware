use std::{collections::HashMap, sync::Arc};

use my_http_server::HttpFailResult;
use rust_extensions::Logger;

use crate::{
    MySignalrActionCallbacks, MySignalrCallbacks, MySignalrCallbacksInstance, MySignalrConnection,
    MySignalrPayloadCallbacks, MySignalrTransportCallbacks, SignalrContractDeserializer,
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

    pub fn add_action<
        TContract: SignalrContractDeserializer<Item = TContract> + Send + Sync + 'static,
        TMySignalrPayloadCallbacks: MySignalrActionCallbacks<TContract, TCtx = TCtx> + Send + Sync + 'static,
    >(
        &mut self,
        action: String,
        callback: TMySignalrPayloadCallbacks,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) {
        if self.actions.contains_key(&action) {
            panic!("Signalr action already registered: {}", action);
        }

        let instance = MySignalrCallbacksInstance {
            action_name: action.to_string(),
            callback: Arc::new(callback),
            logger,
        };

        self.actions.insert(action, Arc::new(instance));
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
        signalr_connection: Arc<MySignalrConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        action_name: String,
        data: Vec<u8>,
        #[cfg(feature = "my-telemetry")] ctx: &mut crate::SignalRTelemetry,
    ) {
        if let Some(action) = self.actions.get(action_name.as_str()) {
            action
                .on(
                    &signalr_connection,
                    headers,
                    &action_name,
                    &data,
                    #[cfg(feature = "my-telemetry")]
                    ctx,
                )
                .await;
        }
    }
}
