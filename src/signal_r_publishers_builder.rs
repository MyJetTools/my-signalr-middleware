use std::sync::Arc;

use crate::{SignalrConnectionsList, SignalrContractSerializer, SignalrMessagePublisher};

pub struct SignalRPublshersBuilder<TCtx: Send + Sync + Default + 'static> {
    signalr_list: Arc<SignalrConnectionsList<TCtx>>,
}

impl<TCtx: Send + Sync + Default + 'static> SignalRPublshersBuilder<TCtx> {
    pub fn new(signalr_list: Arc<SignalrConnectionsList<TCtx>>) -> Self {
        Self { signalr_list }
    }
    pub fn get_publisher<TContract: SignalrContractSerializer + Send + Sync + 'static>(
        &self,
        action_name: String,
    ) -> SignalrMessagePublisher<TContract, TCtx> {
        return SignalrMessagePublisher::new(action_name, self.signalr_list.clone());
    }
}
