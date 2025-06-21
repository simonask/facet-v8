#![doc = include_str!("../README.md")]

mod marshal;

#[cfg(feature = "deno")]
mod extension;

pub use marshal::*;
