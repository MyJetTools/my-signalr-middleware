use std::{collections::HashMap, sync::Arc};

use rust_extensions::Logger;

use crate::{MySignalrConnection, MySignalrPayloadCallbacks};

pub trait SignalrContractDeserializer {
    type Item;
    fn deserialize(data: &[&[u8]]) -> Result<Self::Item, String>;
}

#[async_trait::async_trait]
pub trait MySignalrActionCallbacks<
    TContract: SignalrContractDeserializer<Item = TContract> + Send + Sync + 'static,
>
{
    type TCtx: Send + Sync + Default + 'static;
    async fn on(
        &self,
        connection: &Arc<MySignalrConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        data: TContract,
        #[cfg(feature = "my-telemetry")] ctx: &my_telemetry::MyTelemetryContext,
    );
}

pub struct MySignalrCallbacksInstance<
    TContract: SignalrContractDeserializer<Item = TContract> + Send + Sync + 'static,
    TCtx: Send + Sync + Default + 'static,
> {
    pub action_name: String,
    pub callback: Arc<dyn MySignalrActionCallbacks<TContract, TCtx = TCtx> + Send + Sync + 'static>,
    pub logger: Arc<dyn Logger + Send + Sync + 'static>,
}

#[async_trait::async_trait]
impl<
        TContract: SignalrContractDeserializer<Item = TContract> + Send + Sync + 'static,
        TCtx: Send + Sync + Default + 'static,
    > MySignalrPayloadCallbacks for MySignalrCallbacksInstance<TContract, TCtx>
{
    type TCtx = TCtx;

    async fn on(
        &self,
        connection: &Arc<MySignalrConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        action_name: &str,
        data: &[u8],
        #[cfg(feature = "my-telemetry")] ctx: &my_telemetry::MyTelemetryContext,
    ) {
        let mut params = Vec::new();
        for item in my_json::json_reader::array_parser::JsonArrayIterator::new(data) {
            match item {
                Ok(itm) => params.push(itm),
                Err(err) => {
                    let mut ctx = HashMap::new();
                    ctx.insert("action".to_string(), action_name.to_string());
                    ctx.insert(
                        "payload".to_string(),
                        String::from_utf8_lossy(data).to_string(),
                    );
                    self.logger.write_fatal_error(
                        "Signalr payload handler".to_string(),
                        format!("Can read parameters payloads. Err: {:?}", err),
                        Some(ctx),
                    )
                }
            }
        }

        match TContract::deserialize(&params) {
            Ok(contract) => {
                self.callback
                    .on(
                        connection,
                        headers,
                        contract,
                        #[cfg(feature = "my-telemetry")]
                        ctx,
                    )
                    .await;
            }
            Err(err) => {
                let mut ctx = HashMap::new();
                ctx.insert("action".to_string(), action_name.to_string());
                ctx.insert(
                    "payload".to_string(),
                    String::from_utf8_lossy(data).to_string(),
                );
                self.logger.write_fatal_error(
                    "Signalr payload handler".to_string(),
                    format!("Can not deserialize payload. Err: {}", err),
                    Some(ctx),
                )
            }
        }
    }
}
