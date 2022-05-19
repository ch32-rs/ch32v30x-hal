//! A delay driver based on SysTick.
//!
//! The core is provided with a 64-bit downcounter (SysTick) that supports
//! HCLK or HCLK/8(default) as the time base, has a higher priority and can be used
//! for time base after calibration.
//!
//! **NOTE**: CH32V0x series has no mcycle register.

use embedded_hal::blocking::delay::{DelayMs, DelayUs};

use crate::time::Hertz;

#[allow(non_snake_case)]
#[repr(C)]
pub struct SYSTICK {
    CTLR: u32,
    SR: u32,
    CNT: u64,
    CMP: u64,
}

pub const SYSTICK_BASE_ADDR: u32 = 0xE000F000;

/// System timer (SysTick) as a delay provider.
pub struct Delay {
    frequency: u32,
}

impl Delay {
    /// Configures the system timer (SysTick) as a delay provider.
    ///
    /// `frequency` is a frequency of SysTick, HCLK or HCK/8.
    #[inline]
    pub fn new(frequency: Hertz) -> Self {
        Delay {
            frequency: frequency.raw(),
        }
    }

    /// Delay using the Cortex-M systick for a certain duration, in µs.
    #[allow(clippy::missing_inline_in_public_items)]
    pub fn delay_us(&mut self, us: u32) {
        let mut systick = unsafe { &mut *(SYSTICK_BASE_ADDR as *mut SYSTICK) };

        systick.SR &= !(1 << 0);
        let i = (us as u64) * (self.frequency as u64) / 1_000_000;
        systick.CMP = i;
        systick.CTLR |= 0b110001;

        while systick.SR & 0b1 != 1 {}
        systick.CTLR &= !(1 << 0);
    }

    /// Delay using the Cortex-M systick for a certain duration, in ms.
    #[inline]
    pub fn delay_ms(&mut self, ms: u32) {
        let mut systick = unsafe { &mut *(SYSTICK_BASE_ADDR as *mut SYSTICK) };

        systick.SR &= !(1 << 0);
        let i = (ms as u64) * (self.frequency as u64) / 1_000;
        systick.CMP = i as u64;
        systick.CTLR |= 0b110001;

        while systick.SR & 0b1 != 1 {}
        systick.CTLR &= !(1 << 0);
    }
}

impl DelayMs<u32> for Delay {
    #[inline]
    fn delay_ms(&mut self, ms: u32) {
        Delay::delay_ms(self, ms);
    }
}

// This is a workaround to allow `delay_ms(42)` construction without specifying a type.
impl DelayMs<i32> for Delay {
    #[inline(always)]
    fn delay_ms(&mut self, ms: i32) {
        assert!(ms >= 0);
        Delay::delay_ms(self, ms as u32);
    }
}

impl DelayMs<u16> for Delay {
    #[inline(always)]
    fn delay_ms(&mut self, ms: u16) {
        Delay::delay_ms(self, u32::from(ms));
    }
}

impl DelayMs<u8> for Delay {
    #[inline(always)]
    fn delay_ms(&mut self, ms: u8) {
        Delay::delay_ms(self, u32::from(ms));
    }
}

impl DelayUs<u32> for Delay {
    #[inline]
    fn delay_us(&mut self, us: u32) {
        Delay::delay_us(self, us);
    }
}

// This is a workaround to allow `delay_us(42)` construction without specifying a type.
impl DelayUs<i32> for Delay {
    #[inline(always)]
    fn delay_us(&mut self, us: i32) {
        assert!(us >= 0);
        Delay::delay_us(self, us as u32);
    }
}

impl DelayUs<u16> for Delay {
    #[inline(always)]
    fn delay_us(&mut self, us: u16) {
        Delay::delay_us(self, u32::from(us))
    }
}

impl DelayUs<u8> for Delay {
    #[inline(always)]
    fn delay_us(&mut self, us: u8) {
        Delay::delay_us(self, u32::from(us))
    }
}
