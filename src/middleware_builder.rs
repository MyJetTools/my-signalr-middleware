use std::sync::Arc;

use crate::{
    my_signal_r_actions::MySignalrActions, MySignalrMiddleware, MySignalrPayloadCallbacks,
    MySignalrTransportCallbacks, SignalrContractSerializer, SignalrList,
};

pub struct MiddlewareBuilder<TCtx: Send + Sync + Default + 'static> {
    hub_name: String,
    signalr_list: Arc<SignalrList<TCtx>>,
    actions: MySignalrActions<TCtx>,
}

impl<TCtx: Send + Sync + Default + 'static> MiddlewareBuilder<TCtx> {
    pub fn new(hub_name: String, signalr_list: Arc<SignalrList<TCtx>>) -> Self {
        Self {
            hub_name,
            signalr_list,
            actions: MySignalrActions::new(),
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
        TContract: SignalrContractSerializer + Send + Sync + 'static,
        TMySignalrPayloadCallbacks: MySignalrPayloadCallbacks<TCtx = TCtx> + Send + Sync + 'static,
    >(
        mut self,
        action_name: String,
        action: TMySignalrPayloadCallbacks,
    ) -> Self {
        self.actions.add_action(action_name, action);
        self
    }

    pub fn build(self) -> MySignalrMiddleware<TCtx> {
        MySignalrMiddleware::new(self.hub_name.as_str(), self.signalr_list, self.actions)
    }
}
