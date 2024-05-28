#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

#[rtic::app(device = stm32f1xx_hal::pac)]
mod app {
    use defmt::info;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        info!("echo: init");
        (Shared {}, Local {})
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        info!("echo: idle");

        loop {
            cortex_m::asm::nop();
        }
    }
}
