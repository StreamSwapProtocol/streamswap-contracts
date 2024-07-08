extern crate core;

pub use crate::error::ContractError;
pub mod contract;
mod error;
mod helpers;
mod killswitch;
pub mod state;
#[cfg(test)]
mod tests;
