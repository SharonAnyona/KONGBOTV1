use candid::CandidType;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_http_certification::{HttpRequest, HttpResponse};
use ic_stable_structures::{
    reader::{BufferedReader, Reader},
    writer::{BufferedWriter, Writer},
};
use ic_cdk::api::management_canister::main::raw_rand;

use std::time::Duration;
use ic_cdk_timers::set_timer_interval;
use crate::router::commands::trade_engine::check_pending_orders;
use memory::get_upgrades_memory;
use serde::{Deserialize, Serialize};
use state::State;

pub mod memory;
pub mod router;
pub mod state;

const READER_WRITER_BUFFER_SIZE: usize = 1024 * 1024; // 1MB

#[init]
fn init(args: InitOrUpgradeArgs) {
    let state = State::new(args.oc_public_key);
    state::init(state);
    
    // Initialize alert monitoring
    router::commands::init_alert_monitoring();

    // Set up a timer to check pending orders every 5 minutes
    set_timer_interval(Duration::from_secs(300), || {
        ic_cdk::spawn(async {
            check_pending_orders().await;
        });
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    let mut memory = get_upgrades_memory();
    let writer = BufferedWriter::new(READER_WRITER_BUFFER_SIZE, Writer::new(&mut memory, 0));
    let mut serializer = rmp_serde::Serializer::new(writer).with_struct_map();
    
    let state = state::take();
    state.serialize(&mut serializer).unwrap()
}

#[post_upgrade]
fn post_upgrade(args: InitOrUpgradeArgs) {
    let memory = get_upgrades_memory();
    let reader = BufferedReader::new(READER_WRITER_BUFFER_SIZE, Reader::new(&memory, 0));
    let mut deserializer = rmp_serde::Deserializer::new(reader);
    
    let mut state = State::deserialize(&mut deserializer).unwrap();
    state.update(args.oc_public_key);
    state::init(state);
    
    // Re-initialize alert monitoring after upgrade
    router::commands::init_alert_monitoring();
}

#[query]
async fn http_request(request: HttpRequest) -> HttpResponse {
    router::handle(request, true).await
}

#[update]
async fn http_request_update(request: HttpRequest) -> HttpResponse {
    router::handle(request, false).await
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct InitOrUpgradeArgs {
    pub oc_public_key: String,
}

#[ic_cdk::heartbeat]
async fn heartbeat() {
    // This function is called periodically by the IC runtime
    
    // Generate some randomness to avoid all canisters checking at the same time
    let result = raw_rand().await;
    let random_bytes = match result {
        Ok((bytes,)) => bytes,
        Err(e) => {
            ic_cdk::println!("Failed to get randomness: {:?}", e);
            return;
        }
    };
    
    // Use the first byte as a simple random value (0-255)
    let random_value = random_bytes[0] as u32;
    
    // Only check alerts approximately once every 5 minutes (300 seconds)
    // The IC heartbeat is called roughly every few seconds
    // We'll use the random value to spread out the checks
    if random_value % 60 == 0 {  // Roughly once every 60 heartbeats
        ic_cdk::println!("Heartbeat triggered alert check");
        router::commands::alert::check_alerts().await;
    }
}