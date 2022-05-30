//! GPIO and Alternate function
//!
//! Pin can be configured as:
//! - Floating Input
//! - PullUp Input
//! - PullDown Input
//! - Analog Input
//! - OpenDrain Output
//! - PushPull Output
//! - Alternate Function (input or output)
//!
//! Power On: Floating Input except for some Alternate Function

// CH32V's GPIO is not toggleable, ToggleableOutputPin.
use crate::hal::digital::v2::{InputPin, OutputPin, PinState, StatefulOutputPin};
use crate::pac::{GPIOA, GPIOB, GPIOC, GPIOD, GPIOE};
use crate::rcc::rec::ResetEnable;

use core::convert::Infallible;
use core::marker::PhantomData;

mod convert;
pub use convert::PinMode;

/// Extension trait to split a GPIO peripheral into independent pins and
/// registers
pub trait GpioExt {
    /// The parts to split the GPIO into
    type Parts;

    /// The Reset and Enable control block for this GPIO block
    type Rec: ResetEnable;

    /// Takes the GPIO peripheral and splits it into Zero-Sized Types
    /// (ZSTs) representing individual pins. These are public
    /// members of the return type.
    ///
    /// ```
    /// let device_peripherals = ch32xxx::Peripherals.take().unwrap();
    /// let ccdr = ...; // From RCC
    ///
    /// let gpioa = device_peripherals.GPIOA.split(ccdr.peripheral.GPIOA);
    ///
    /// let pa0 = gpioa.pa0; // Pin 0
    /// ```
    fn split(self, prec: Self::Rec) -> Self::Parts;
}

/// Id, port and mode for any pin
pub trait PinExt {
    /// Current pin mode
    type Mode;
    /// Pin number
    fn pin_id(&self) -> u8;
    /// Port number starting from 0
    fn port_id(&self) -> u8;
}

/// Some alternate mode (type state)
pub struct Alternate<Otype = PushPull>(PhantomData<Otype>);

/// Input mode (type state)
pub struct Input<MODE = Floating>(PhantomData<MODE>);

/// Floating input (type state)
pub struct Floating;

/// Pulled down input (type state)
pub struct PullDown;

/// Pulled up input (type state)
pub struct PullUp;

/// Output mode (type state)
pub struct Output<MODE = PushPull> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
pub struct PushPull;

/// Open drain input or output (type state)
pub struct OpenDrain;

/// Analog mode (type state)
pub struct Analog;

/// JTAG/SWD mote (type state)
pub type Debugger = Alternate<PushPull>;

mod marker {
    // /// Marker trait that show if `ExtiPin` can be implemented
    // pub trait Interruptable {}
    /// Marker trait for readable pin modes
    pub trait Readable {}
    /// Marker trait for slew rate configurable pin modes
    pub trait OutputSpeed {}
    /// Marker trait for active pin modes
    pub trait Active {}
    /// Marker trait for all pin modes except alternate
    pub trait NotAlt {}
}

// impl<MODE> marker::Interruptable for Output<MODE> {}
// impl marker::Interruptable for Input {}
impl<IType> marker::Readable for Input<IType> {}
impl marker::Readable for Output<OpenDrain> {}
impl<IType> marker::Active for Input<IType> {}
impl<Otype> marker::OutputSpeed for Output<Otype> {}
impl<Otype> marker::OutputSpeed for Alternate<Otype> {}
impl<Otype> marker::Active for Output<Otype> {}
impl<Otype> marker::Active for Alternate<Otype> {}
impl<IType> marker::NotAlt for Input<IType> {}
impl<Otype> marker::NotAlt for Output<Otype> {}
impl marker::NotAlt for Analog {}

/// GPIO Pin speed selection
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Speed {
    /// Low speed, 2MHz
    Low = 0b10,
    /// Medium speed, 10MHz
    Medium = 0b01,
    /// High speed, 50MHz
    High = 0b11,
}

/// Generic pin type
///
/// - `MODE` is one of the pin modes (see [Modes](crate::gpio#modes) section).
/// - `P` is port name: `A` for GPIOA, `B` for GPIOB, etc.
/// - `N` is pin number: from `0` to `15`.
pub struct Pin<const P: char, const N: u8, MODE = Analog> {
    _mode: PhantomData<MODE>,
}
impl<const P: char, const N: u8, MODE> Pin<P, N, MODE> {
    const fn new() -> Self {
        Self { _mode: PhantomData }
    }
}

impl<const P: char, const N: u8, MODE> PinExt for Pin<P, N, MODE> {
    type Mode = MODE;

    #[inline(always)]
    fn pin_id(&self) -> u8 {
        N
    }
    #[inline(always)]
    fn port_id(&self) -> u8 {
        P as u8 - b'A'
    }
}

impl<const P: char, const N: u8, MODE> Pin<P, N, MODE>
where
    MODE: marker::OutputSpeed,
{
    /// Set pin speed
    pub fn set_speed(&mut self, speed: Speed) {
        let offset = 4 * { N } % 32;

        // lower ports
        if N < 8 {
            unsafe {
                (*Gpio::<P>::ptr()).cfglr.modify(|r, w| {
                    w.bits((r.bits() & !(0b11 << offset)) | ((speed as u32) << offset))
                });
            }
        } else {
            unsafe {
                (*Gpio::<P>::ptr()).cfghr.modify(|r, w| {
                    w.bits((r.bits() & !(0b11 << offset)) | ((speed as u32) << offset))
                });
            }
        }
    }

    /// Set pin speed
    pub fn speed(mut self, speed: Speed) -> Self {
        self.set_speed(speed);
        self
    }
}

// TODO: erased pin

impl<const P: char, const N: u8, MODE> Pin<P, N, MODE> {
    /// Set the output of the pin regardless of its mode.
    /// Primarily used to set the output value of the pin
    /// before changing its mode to an output to avoid
    /// a short spike of an incorrect value
    #[inline(always)]
    fn _set_state(&mut self, state: PinState) {
        match state {
            PinState::High => self._set_high(),
            PinState::Low => self._set_low(),
        }
    }
    #[inline(always)]
    fn _set_high(&mut self) {
        // NOTE(unsafe) atomic write to a stateless register
        unsafe { (*Gpio::<P>::ptr()).bshr.write(|w| w.bits(1 << N)) }
    }
    #[inline(always)]
    fn _set_low(&mut self) {
        // NOTE(unsafe) atomic write to a stateless register
        unsafe { (*Gpio::<P>::ptr()).bshr.write(|w| w.bits(1 << (16 + N))) }
    }
    #[inline(always)]
    fn _is_set_low(&self) -> bool {
        // NOTE(unsafe) atomic read with no side effects
        unsafe { (*Gpio::<P>::ptr()).outdr.read().bits() & (1 << N) == 0 }
    }
    #[inline(always)]
    fn _is_low(&self) -> bool {
        // NOTE(unsafe) atomic read with no side effects
        unsafe { (*Gpio::<P>::ptr()).indr.read().bits() & (1 << N) == 0 }
    }
}

/// Gpio port helper
struct Gpio<const P: char>;
impl<const P: char> Gpio<P> {
    const fn ptr() -> *const crate::pac::gpioa::RegisterBlock {
        match P {
            'A' => crate::pac::GPIOA::ptr(),
            'B' => crate::pac::GPIOB::ptr(),
            'C' => crate::pac::GPIOC::ptr(),
            'D' => crate::pac::GPIOD::ptr(),
            'E' => crate::pac::GPIOE::ptr(),
            _ => panic!("Unknown GPIO port"),
        }
    }
}
