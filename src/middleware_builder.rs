use std::sync::Arc;

use rust_extensions::Logger;

use crate::{
    my_signal_r_actions::MySignalrActions, MySignalrActionCallbacks, MySignalrMiddleware,
    MySignalrTransportCallbacks, SignalrConnectionsList, SignalrContractDeserializer,
};

pub struct MiddlewareBuilder<TCtx: Send + Sync + Default + 'static> {
    hub_name: String,
    signal_r_list: Arc<SignalrConnectionsList<TCtx>>,
    actions: MySignalrActions<TCtx>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
    disconnect_timeout: std::time::Duration,
}

impl<TCtx: Send + Sync + Default + 'static> MiddlewareBuilder<TCtx> {
    pub fn new(
        hub_name: String,
        signalr_list: Arc<SignalrConnectionsList<TCtx>>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        Self {
            hub_name,
            signal_r_list: signalr_list,
            actions: MySignalrActions::new(),
            logger,
            disconnect_timeout: std::time::Duration::from_secs(60),
        }
    }

    pub fn set_disconnect_timeout(mut self, disconnect_timeout: std::time::Duration) -> Self {
        self.disconnect_timeout = disconnect_timeout;
        self
    }

    pub fn with_transport_callback(
        mut self,
        transport_callback: Arc<
            dyn MySignalrTransportCallbacks<TCtx = TCtx> + Send + Sync + 'static,
        >,
    ) -> Self {
        if self.actions.transport_callbacks.is_some() {
            panic!("Transport callback is already registered");
        }

        self.actions.transport_callbacks = Some(transport_callback);
        self
    }

    pub fn with_action<
        TContract: SignalrContractDeserializer<Item = TContract> + Send + Sync + 'static,
        TMySignalrPayloadCallbacks: MySignalrActionCallbacks<TContract, TCtx = TCtx> + Send + Sync + 'static,
    >(
        mut self,
        action_name: String,
        action: TMySignalrPayloadCallbacks,
    ) -> Self {
        self.actions
            .add_action(action_name, action, self.logger.clone());
        self
    }

    pub fn build(self) -> MySignalrMiddleware<TCtx> {
        MySignalrMiddleware::new(
            self.hub_name.as_str(),
            self.signal_r_list,
            self.actions,
            self.disconnect_timeout,
        )
    }
}
