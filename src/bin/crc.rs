#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

#[rtic::app(device = stm32f1xx_hal::pac)]
mod app {
    use defmt::info;
    use stm32f1xx_hal::prelude::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let dp = cx.device;
        let mut crc = dp.CRC.new();

        crc.reset();
        crc.write(0x12345678);

        let val = crc.read();
        info!("found={:08x}, expected={:08x}", val, 0xdf8a8a2b_u32);
        (Shared {}, Local {})
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}
