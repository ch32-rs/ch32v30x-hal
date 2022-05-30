use super::*;

impl<const P: char, const N: u8, MODE: PinMode> Pin<P, N, MODE> {
    /// Puts `self` into mode `M`.
    ///
    /// This violates the type state constraints from `MODE`, so callers must
    /// ensure they use this properly.
    #[inline(always)]
    pub(super) fn mode<M: PinMode>(&mut self) {
        let offset = 4 * N % 8;
        unsafe {
            if MODE::CFGR != M::CFGR {
                if N < 8 {
                    (*Gpio::<P>::ptr()).cfglr.modify(|r, w| {
                        w.bits(r.bits() & !(0b1111 << offset) | (M::CFGR << offset))
                    });
                } else {
                    (*Gpio::<P>::ptr()).cfghr.modify(|r, w| {
                        w.bits(r.bits() & !(0b1111 << offset) | (M::CFGR << offset))
                    });
                }
            }

            // PullUp/PullDown is controlled by the BCR(down) / BSHR(up) register.
            // seems undocumented, but it's in the hal library.
            if MODE::PULL_DOWN {
                (*Gpio::<P>::ptr()).bcr.write(|w| w.bits(0b1 << N));
            } else if MODE::PULL_UP {
                (*Gpio::<P>::ptr()).bshr.write(|w| w.bits(0b1 << N));
            }
        }
    }
}

/// Marker trait for valid pin modes (type state).
///
/// This trait is sealed and cannot be implemented by outside types
pub trait PinMode: crate::Sealed {
    // These constants are used to implement the pin configuration code.
    // They are not part of public API.

    #[doc(hidden)]
    const CFGR: u32 = u32::MAX; // <<CNF:2, MODE:2>>
    #[doc(hidden)]
    const PULL_DOWN: bool = false;
    #[doc(hidden)]
    const PULL_UP: bool = false;
}

impl<IType> crate::Sealed for Input<IType> {}
impl PinMode for Input<Floating> {
    const CFGR: u32 = 0b01_00;
}
impl PinMode for Input<PullDown> {
    const CFGR: u32 = 0b10_00;
    const PULL_DOWN: bool = true;
}
impl PinMode for Input<PullUp> {
    const CFGR: u32 = 0b10_00;
    const PULL_UP: bool = true;
}

impl crate::Sealed for Analog {}
impl PinMode for Analog {
    const CFGR: u32 = 0b00_00;
}

// use 0b10 for low speed, 2MHz max
impl<Otype> crate::Sealed for Output<Otype> {}
impl PinMode for Output<PushPull> {
    const CFGR: u32 = 0b10_00;
}
impl PinMode for Output<OpenDrain> {
    const CFGR: u32 = 0b10_01;
}

impl<Otype> crate::Sealed for Alternate<Otype> {}
impl PinMode for Alternate<PushPull> {
    const CFGR: u32 = 0b10_10;
}
impl PinMode for Alternate<OpenDrain> {
    const CFGR: u32 = 0b10_11;
}
