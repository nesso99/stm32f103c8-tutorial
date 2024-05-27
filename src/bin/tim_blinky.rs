#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use stm32f1xx_hal::{gpio::GpioExt, pac::Peripherals, prelude::*};
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut gpioc = dp.GPIOC.split();

    let clocks = rcc.cfgr.use_hse(8.MHz()).freeze(&mut flash.acr);
    let mut delay = dp.TIM2.delay_us(&clocks);
    let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    loop {
        info!("toggle");
        pc13.toggle();
        delay.delay(1.secs());
    }
}
