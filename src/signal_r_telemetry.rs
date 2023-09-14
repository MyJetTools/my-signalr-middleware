use my_telemetry::TelemetryEventTagsBuilder;
use rust_extensions::StrOrString;

pub struct SignalRTelemetry {
    ctx: my_telemetry::MyTelemetryContext,
    pub tags: TelemetryEventTagsBuilder,
}

impl SignalRTelemetry {
    pub fn new(ctx: my_telemetry::MyTelemetryContext) -> Self {
        Self {
            ctx,
            tags: TelemetryEventTagsBuilder::new(),
        }
    }
    pub fn get_ctx(&self) -> &my_telemetry::MyTelemetryContext {
        &self.ctx
    }

    pub fn add_tag(
        &mut self,
        key: impl Into<StrOrString<'static>>,
        value: impl Into<StrOrString<'static>>,
    ) {
        self.tags.add_as_ref(key, value);
    }
}
