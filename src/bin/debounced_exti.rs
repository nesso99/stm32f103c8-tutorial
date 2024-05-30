#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true)]
mod app {
    use defmt::{info, Format};
    use stm32f1xx_hal::{
        gpio::{Edge, ExtiPin, Input, Output, PullUp, PushPull, PB0, PC13},
        pac::{Interrupt, NVIC, TIM2},
        prelude::*,
        timer::DelayMs,
    };

    #[derive(Debug, PartialEq)]
    enum ButtonState {
        Pressed,
        Released,
    }

    struct ButtonHandler {
        pub state: ButtonState,
    }

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        button: PB0<Input<PullUp>>,
        delay_handler: DelayMs<TIM2>,
        button_handler: ButtonHandler,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut dp = cx.device;
        let mut flash = dp.FLASH.constrain();
        let rcc = dp.RCC.constrain();
        let mut gpiob = dp.GPIOB.split();
        let mut gpioc = dp.GPIOC.split();
        let mut afio = dp.AFIO.constrain();

        let clocks = rcc.cfgr.use_hse(8.MHz()).freeze(&mut flash.acr);
        let delay_handler = dp.TIM2.delay_ms(&clocks);

        let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        pc13.set_high();

        let mut button = gpiob.pb0.into_pull_up_input(&mut gpiob.crl);
        button.enable_interrupt(&mut dp.EXTI);
        button.make_interrupt_source(&mut afio);
        button.trigger_on_edge(&mut dp.EXTI, Edge::RisingFalling);

        (
            Shared {},
            Local {
                led: pc13,
                button,
                delay_handler,
                button_handler: ButtonHandler {
                    state: ButtonState::Released,
                },
            },
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = EXTI0, priority=1, local = [led, button, delay_handler, button_handler])]
    fn exti0(cx: exti0::Context) {
        let exti0::LocalResources {
            button,
            button_handler,
            led,
            delay_handler,
            ..
        } = cx.local;

        delay_handler.delay_ms(20_u16);
        button.clear_interrupt_pending_bit();
        NVIC::unpend(Interrupt::EXTI0);

        if button.is_low() {
            button_handler.state = ButtonState::Pressed;
        } else {
            led.toggle();
            button_handler.state = ButtonState::Released;
        }

        info!("{:?}", button_handler.state);
    }

    impl Format for ButtonState {
        fn format(&self, f: defmt::Formatter<'_>) {
            match self {
                ButtonState::Pressed => defmt::write!(f, "Pressed"),
                ButtonState::Released => defmt::write!(f, "Released"),
            }
        }
    }
}
