#![no_std]
#![allow(non_camel_case_types, non_snake_case)]

pub use embedded_hal as hal;
pub use nb;
pub use nb::block;

// pub use embedded_time as time;

pub use ch32v3 as pac;

/// Enable use of interrupt macro.
#[cfg(feature = "rt")]
pub use crate::pac::interrupt;

pub mod delay;
pub mod prelude;
pub mod time;

pub mod gpio;
pub mod rcc;

mod sealed {
    pub trait Sealed {}
}
pub(crate) use sealed::Sealed;
