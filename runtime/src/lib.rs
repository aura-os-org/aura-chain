#![cfg_attr(not(feature = "std"), no_std)]

pub use sp_runtime::BuildStorage;

// Базовая структура runtime
pub struct Runtime;

impl Runtime {
    pub fn new() -> Self {
        Runtime
    }
}
