//! Peripheral Reset and Enable Control (REC)

use riscv::interrupt;

use crate::pac::{EXTEND, RCC};
use core::marker::PhantomData;

use super::Rcc;

/// A trait for Resetting, Enabling and Disabling a single peripheral
pub trait ResetEnable {
    /// Enable this peripheral
    #[allow(clippy::return_self_not_must_use)]
    fn enable(self) -> Self;
    /// Disable this peripheral
    #[allow(clippy::return_self_not_must_use)]
    fn disable(self) -> Self;
    /// Reset this peripheral
    #[allow(clippy::return_self_not_must_use)]
    fn reset(self) -> Self;
}

// TODO PeripheralREC
/*
pub struct PeripheralREC {
    pub GPIOA: Gpioa,
    pub GPIOB: Gpiob,
}

impl PeripheralREC {
    pub(super) unsafe fn new_singleton() -> PeripheralREC {
        PeripheralREC {
            GPIOA: Gpioa {
                _marker: PhantomData,
            },
            GPIOB: Gpiob {
                _marker: PhantomData,
            },
        }
    }
}
 */

macro_rules! peripheral_reset_and_enable_control_gen {
    ($($PERIPH:ident: $Periph:ident => ($enr:ident, $enf:ident, $rstr:ident, $rstf:ident) ; )+) => {
        pub struct PeripheralREC {
            $(pub $PERIPH: $Periph,)*
        }

        impl PeripheralREC {
            pub(super) unsafe fn new_singleton() -> PeripheralREC {
                PeripheralREC {
                    $(
                        $PERIPH: $Periph {
                            _marker: PhantomData,
                        },
                    )*
                }
            }
        }

        // Impl Periph
        $(
            pub struct $Periph {
                _marker: PhantomData<*const ()>,
            }

            unsafe impl Send for $Periph {}

            impl ResetEnable for $Periph {
                #[inline(always)]
                fn enable(self) -> Self {
                    interrupt::free(|_| {
                        let enr = unsafe { &(*RCC::ptr()).$enr };
                        enr.modify(|_, w| w.$enf().set_bit())
                    });
                    self
                }

                #[inline(always)]
                fn disable(self) -> Self {
                    interrupt::free(|_| {
                        let enr = unsafe { &(*RCC::ptr()).$enr };
                        enr.modify(|_, w| w.$enf().clear_bit())
                    });
                    self
                }

                #[inline(always)]
                fn reset(self) -> Self {
                    interrupt::free(|_| {
                        let rstr = unsafe { &(*RCC::ptr()).$rstr };
                        rstr.modify(|_, w| w.$rstf().set_bit())
                    });
                    self
                }
            }
        )*
    };
}

peripheral_reset_and_enable_control_gen!(
    GPIOA: Gpioa => (apb2pcenr, iopaen, apb2prstr, ioparst) ;
    GPIOB: Gpiob => (apb2pcenr, iopben, apb2prstr, iopbrst) ;
    GPIOC: Gpioc => (apb2pcenr, iopcen, apb2prstr, iopcrst) ;
    GPIOD: Gpiod => (apb2pcenr, iopden, apb2prstr, iopdrst) ;
    // GPIOE: Gpioe => (apb2pcenr, iopeen, apb2prstr, ioperst) ;

    AFIO: Afio => (apb2pcenr, afioen, apb2prstr, afiorst) ;

    USART1: Usart1 => (apb2pcenr, usart1en, apb2prstr, usart1rst) ;
);
