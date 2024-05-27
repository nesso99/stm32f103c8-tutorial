#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use defmt::info;
use stm32f1xx_hal::{gpio::GpioExt, pac::Peripherals};
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

    let mut gpioc = dp.GPIOC.split();
    let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    loop {
        info!("toggle");
        pc13.toggle();
        for _ in 0..500_000 {
            nop();
        }
    }
}
