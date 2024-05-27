#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use defmt::info;
#[allow(unused_imports)]
use stm32f1xx_hal::pac::interrupt;
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    loop {
        info!("echo");
        for _ in 0..500_000 {
            nop();
        }
    }
}
