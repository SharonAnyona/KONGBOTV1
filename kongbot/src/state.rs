use crate::router::commands::alert::PriceAlert;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
// use std::collections::HashMap;
use oc_bots_sdk::types::Chat;

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Serialize, Deserialize)]
pub struct State {
    oc_public_key: String,
    // Add a field to store price alerts
    #[serde(default)]
    price_alerts: Vec<PriceAlert>,
}

const STATE_ALREADY_INITIALIZED: &str = "State has already been initialized";
const STATE_NOT_INITIALIZED: &str = "State has not been initialized";

pub fn mutate<F: FnOnce(&mut State) -> R, R>(f: F) -> R {
    STATE.with_borrow_mut(|s| f(s.as_mut().expect(STATE_NOT_INITIALIZED)))
}

pub fn read<F: FnOnce(&State) -> R, R>(f: F) -> R {
    STATE.with_borrow(|s| f(s.as_ref().expect(STATE_NOT_INITIALIZED)))
}

pub fn init(state: State) {
    STATE.with_borrow_mut(|s| {
        if s.is_some() {
            panic!("{}", STATE_ALREADY_INITIALIZED);
        } else {
            *s = Some(state);
        }
    })
}

pub fn take() -> State {
    STATE.take().expect(STATE_NOT_INITIALIZED)
}

impl State {
    pub fn new(oc_public_key: String) -> State {
        State { 
            oc_public_key,
            price_alerts: Vec::new(),
        }
    }

    pub fn update(&mut self, oc_public_key: String) {
        self.oc_public_key = oc_public_key;
    }

    pub fn oc_public_key(&self) -> &str {
        &self.oc_public_key
    }
    
    // Add methods to manage price alerts
    pub fn add_price_alert(&mut self, alert: PriceAlert) {
        self.price_alerts.push(alert);
    }
    
    pub fn get_price_alerts(&self) -> Vec<PriceAlert> {
        self.price_alerts.clone()
    }
    
    pub fn mark_alert_triggered(&mut self, chat_id: Chat, coin: &str) {
        for alert in &mut self.price_alerts {
            if alert.chat_id == chat_id && alert.coin == coin {
                alert.triggered = true;
            }
        }
    }
    
    pub fn remove_alert(&mut self, chat_id: Chat, coin: &str) -> bool {
        let len_before = self.price_alerts.len();
        self.price_alerts.retain(|alert| 
            !(alert.chat_id == chat_id && alert.coin == coin)
        );
        len_before > self.price_alerts.len()
    }
}
