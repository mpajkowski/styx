#![no_std]

#[cfg(test)]
extern crate std;

mod config;
pub(crate) mod parser;

pub use config::Config;
