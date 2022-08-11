use std::sync::Arc;

use hyper::Method;
use my_http_server::{
    HttpContext, HttpFailResult, HttpOkResult, HttpOutput, HttpServerMiddleware,
    HttpServerRequestFlow, RequestData, WebContentType,
};
use tokio::sync::Mutex;

use crate::{signal_r_list::SignalrList, MySignalrCallbacks, WebSocketCallbacks};

pub struct MySignalrMiddleware {
    pub hub_name: String,
    negotiate_uri: String,
    socket_id: Mutex<i64>,
    web_socket_callback: Arc<WebSocketCallbacks>,
    signalr_list: Arc<SignalrList>,
    my_signal_r_callbacks: Arc<dyn MySignalrCallbacks + Send + Sync + 'static>,
}

impl MySignalrMiddleware {
    pub fn new(
        hub_name: &str,
        my_signal_r_callbacks: Arc<dyn MySignalrCallbacks + Send + Sync + 'static>,
    ) -> Self {
        let hub_name = if hub_name.starts_with('/') {
            hub_name.to_string()
        } else {
            format!("/{}", hub_name)
        };

        let signalr_list = Arc::new(SignalrList::new());

        Self {
            negotiate_uri: compile_negotiate_uri(hub_name.as_str()),
            signalr_list: signalr_list.clone(),
            hub_name,
            web_socket_callback: Arc::new(WebSocketCallbacks {
                signalr_list,
                my_signal_r_callbacks: my_signal_r_callbacks.clone(),
            }),
            socket_id: Mutex::new(0),

            my_signal_r_callbacks,
        }
    }

    async fn get_socket_id(&self) -> i64 {
        let mut socket_no = self.socket_id.lock().await;
        *socket_no += 1;
        *socket_no
    }

    async fn handle_negotiate_request(
        &self,
        _ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let (_, response) =
            crate::process_connect(&self.my_signal_r_callbacks, &self.signalr_list, None).await;
        HttpOutput::Content {
            headers: None,
            content_type: Some(WebContentType::Text),
            content: response.into_bytes(),
        }
        .into_ok_result(true)
        .into()
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for MySignalrMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult> {
        if !ctx
            .request
            .get_path_lower_case()
            .starts_with(self.hub_name.as_str())
        {
            return get_next.next(ctx).await;
        }

        if ctx
            .request
            .get_optional_header("sec-websocket-key")
            .is_some()
        {
            if let RequestData::AsRaw(request) = &mut ctx.request.req {
                let id = self.get_socket_id().await;
                return my_http_server_web_sockets::handle_web_socket_upgrade(
                    request,
                    &self.web_socket_callback,
                    id,
                    ctx.request.addr,
                )
                .await;
            }

            return get_next.next(ctx).await;
        }

        if ctx.request.method == Method::POST {
            if ctx.request.get_path_lower_case() == self.negotiate_uri.as_str() {
                return self.handle_negotiate_request(ctx).await;
            }
        }

        get_next.next(ctx).await
    }
}

fn compile_negotiate_uri(hub_name: &str) -> String {
    let mut result = Vec::new();

    result.extend_from_slice(hub_name.to_lowercase().as_bytes());

    if !hub_name.ends_with('/') {
        result.push(b'/');
    }

    result.extend_from_slice("negotiate".as_bytes());

    String::from_utf8(result).unwrap()
}
