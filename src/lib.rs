#![no_std]
#![allow(non_camel_case_types)]

pub use embedded_hal as hal;
pub use nb;
pub use nb::block;

// pub use embedded_time as time;

pub use ch32v3 as pac;

/// Enable use of interrupt macro.
#[cfg(feature = "rt")]
pub use crate::pac::interrupt;

pub mod time;
pub mod prelude;
pub mod delay;

pub mod rcc;
