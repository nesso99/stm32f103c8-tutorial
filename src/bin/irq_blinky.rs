#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true)]
mod app {
    use stm32f1xx_hal::{
        gpio::{Output, PushPull, PC13},
        pac::TIM2,
        prelude::*,
        timer::{CounterMs, Event},
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        timer_handler: CounterMs<TIM2>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let dp = cx.device;
        let mut flash = dp.FLASH.constrain();
        let rcc = dp.RCC.constrain();
        let mut gpioc = dp.GPIOC.split();

        let clocks = rcc.cfgr.use_hse(8.MHz()).freeze(&mut flash.acr);
        let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        pc13.set_high();

        let mut timer_handler = dp.TIM2.counter_ms(&clocks);
        timer_handler.start(1.secs()).unwrap();

        // Generate an interrupt when the timer expires
        timer_handler.listen(Event::Update);

        (
            Shared {},
            Local {
                led: pc13,
                timer_handler,
            },
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = TIM2, priority = 1, local = [led, timer_handler, count: u8 = 0])]
    fn tick(cx: tick::Context) {
        cx.local.led.toggle();
        *cx.local.count += 1;

        if *cx.local.count == 4 {
            cx.local.timer_handler.start(500.millis()).unwrap();
        } else if *cx.local.count == 12 {
            cx.local.timer_handler.start(1.secs()).unwrap();
            *cx.local.count = 0;
        }

        // Clears the update flag
        cx.local.timer_handler.clear_interrupt(Event::Update);
    }
}
