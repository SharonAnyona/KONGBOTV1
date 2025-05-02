use crate::InitOrUpgradeArgs;
use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct State {
    oc_public_key: String,
}

impl State {
    pub fn new(oc_public_key: String) -> Self {
        Self { oc_public_key }
    }

    pub fn oc_public_key(&self) -> &str {
        &self.oc_public_key
    }
}

pub fn init_with_args(args: InitOrUpgradeArgs) {
    STATE.with(|state| {
        if state.borrow().is_none() {
            *state.borrow_mut() = Some(State::new(args.oc_public_key));
        }
    });
}

pub fn read<F: FnOnce(&State) -> R, R>(f: F) -> R {
    STATE.with(|state| f(state.borrow().as_ref().expect("State not initialized")))
}

pub fn mutate<F: FnOnce(&mut State) -> R, R>(f: F) -> R {
    STATE.with(|state| f(state.borrow_mut().as_mut().expect("State not initialized")))
}

pub fn take() -> State {
    STATE.with(|state| state.take().expect("State not initialized"))
}
