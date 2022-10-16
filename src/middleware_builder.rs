use std::sync::Arc;

use rust_extensions::Logger;

use crate::{
    my_signal_r_actions::MySignalrActions, MySignalrActionCallbacks, MySignalrMiddleware,
    MySignalrTransportCallbacks, SignalrContractDeserializer, SignalrList,
};

pub struct MiddlewareBuilder<TCtx: Send + Sync + Default + 'static> {
    hub_name: String,
    signalr_list: Arc<SignalrList<TCtx>>,
    actions: MySignalrActions<TCtx>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
}

impl<TCtx: Send + Sync + Default + 'static> MiddlewareBuilder<TCtx> {
    pub fn new(
        hub_name: String,
        signalr_list: Arc<SignalrList<TCtx>>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        Self {
            hub_name,
            signalr_list,
            actions: MySignalrActions::new(),
            logger,
        }
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
        MySignalrMiddleware::new(self.hub_name.as_str(), self.signalr_list, self.actions)
    }
}
