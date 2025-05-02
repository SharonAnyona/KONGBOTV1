use ic_http_certification::{HttpRequest as IcHttpRequest, HttpResponse as IcHttpResponse};
use oc_bots_sdk_canister::{HttpMethod::*, HttpRouter};
use std::sync::LazyLock;

pub mod commands;
pub mod definition;
pub mod metrics;

static ROUTER: LazyLock<HttpRouter> = LazyLock::new(init_router);

fn init_router() -> HttpRouter {
    HttpRouter::default()
        .route("/", GET, definition::get)
        .route("/execute_command", POST, commands::execute)
        .route("/metrics", GET, metrics::get)
        .route("/bot_definition", GET, definition::get)
        .fallback(definition::get)
}

pub async fn handle_command(request: IcHttpRequest) -> IcHttpResponse {
    ROUTER.handle(request, false).await
}

pub async fn handle_definition(request: IcHttpRequest) -> IcHttpResponse {
    ROUTER.handle(request, true).await
}
