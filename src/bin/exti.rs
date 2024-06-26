#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true)]
mod app {
    use stm32f1xx_hal::{
        gpio::{Edge, ExtiPin, Input, Output, PullUp, PushPull, PB0, PC13},
        prelude::*,
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        button: PB0<Input<PullUp>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut dp = cx.device;
        let mut flash = dp.FLASH.constrain();
        let rcc = dp.RCC.constrain();
        let mut gpiob = dp.GPIOB.split();
        let mut gpioc = dp.GPIOC.split();
        let mut afio = dp.AFIO.constrain();

        let _clocks = rcc.cfgr.use_hse(8.MHz()).freeze(&mut flash.acr);

        let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        pc13.set_high();

        let mut button = gpiob.pb0.into_pull_up_input(&mut gpiob.crl);
        button.enable_interrupt(&mut dp.EXTI);
        button.make_interrupt_source(&mut afio);
        button.trigger_on_edge(&mut dp.EXTI, Edge::Falling);

        (Shared {}, Local { led: pc13, button })
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = EXTI0, priority = 1, local = [led, button])]
    fn tick(cx: tick::Context) {
        cx.local.led.toggle();
        cx.local.button.clear_interrupt_pending_bit();
    }
}
