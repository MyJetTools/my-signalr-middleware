use std::sync::Arc;

use crate::{MySignalrConnection, SignalRParam, SignalrConnectionsList};

pub trait SignalrContractSerializer {
    fn serialize(self) -> Vec<Vec<u8>>;
}
pub struct SignalrMessagePublisher<
    TContract: SignalrContractSerializer + Send + Sync + 'static,
    TCtx: Default + Send + Sync + 'static,
> {
    signalr_list: Arc<SignalrConnectionsList<TCtx>>,
    itm: std::marker::PhantomData<TContract>,
    action_name: String,
}

impl<
        TContract: SignalrContractSerializer + Send + Sync + 'static,
        TCtx: Default + Send + Sync + 'static,
    > SignalrMessagePublisher<TContract, TCtx>
{
    pub fn new(action_name: String, signalr_list: Arc<SignalrConnectionsList<TCtx>>) -> Self {
        Self {
            action_name,
            signalr_list,
            itm: std::marker::PhantomData,
        }
    }

    pub async fn broadcast_to_all(&self, contract: TContract) {
        if let Some(connections) = self.signalr_list.get_all().await {
            let payload = contract.serialize();

            for connection in connections {
                let params = SignalRParam::Raw(payload.as_slice());

                connection.send(self.action_name.as_str(), &params).await;
            }
        }
    }

    pub async fn send_to_connection(
        &self,
        connection: &MySignalrConnection<TCtx>,
        contract: TContract,
    ) {
        let payload = contract.serialize();
        let params = SignalRParam::Raw(payload.as_slice());
        connection.send(self.action_name.as_str(), &params).await;
    }

    pub async fn send_to_tagged_connections(&self, key: &str, contract: TContract) {
        if let Some(connections) = self.signalr_list.get_tagged_connections(key).await {
            let payload = contract.serialize();

            for connection in connections {
                let params = SignalRParam::Raw(payload.as_slice());
                connection.send(self.action_name.as_str(), &params).await;
            }
        }
    }

    pub async fn send_to_tagged_connections_with_value(
        &self,
        key: &str,
        value: &str,
        contract: TContract,
    ) {
        if let Some(connections) = self
            .signalr_list
            .get_tagged_connections_with_value(key, value)
            .await
        {
            let payload = contract.serialize();

            for connection in connections {
                let params = SignalRParam::Raw(payload.as_slice());
                connection.send(self.action_name.as_str(), &params).await;
            }
        }
    }
}
