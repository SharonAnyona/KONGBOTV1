use candid::{CandidType, Deserialize};
// Only need standard IC types here
use ic_http_certification::{HttpRequest as IcHttpRequest, HttpResponse as IcHttpResponse};

#[derive(CandidType, Deserialize)]
pub struct InitOrUpgradeArgs {
    pub oc_public_key: String,
}

mod memory;
mod router;
mod state;

#[ic_cdk::init]
fn init(args: InitOrUpgradeArgs) {
    state::init_with_args(args);
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: InitOrUpgradeArgs) {
    state::init_with_args(args);
}

#[ic_cdk::update]
async fn bot_command_execute(request: IcHttpRequest) -> IcHttpResponse {
    router::handle_command(request).await
}

#[ic_cdk::query]
async fn http_request(request: IcHttpRequest) -> IcHttpResponse {
    router::handle_definition(request).await
}
