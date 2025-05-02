use oc_bots_sdk_canister::{HttpRequest, HttpResponse};

pub async fn get(_request: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        body: serde_json::to_vec(&serde_json::json!({
            "active_memory": ic_cdk::api::stable::stable_size() * 64 * 1024,
            "heap_memory": ic_cdk::api::canister_balance()
        }))
        .unwrap(),
    }
}
