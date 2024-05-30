#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

use rtic_monotonics::systick::prelude::*;
systick_monotonic!(Mono, 1000);

#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true)]
mod app {
    use crate::Mono;
    use defmt::{info, Format};
    use fugit::MillisDurationU32;
    use rtic_monotonics::Monotonic;
    use stm32f1xx_hal::{
        gpio::{Edge, ExtiPin, Input, Output, PullUp, PushPull, PB0, PC13},
        pac::{Interrupt, NVIC, TIM2},
        prelude::*,
        timer::DelayMs,
    };

    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum ButtonState {
        None,
        Pressed,
        ReleasedSinglePending,
        PressedDoublePending,
    }

    // #[derive(Debug, PartialEq)]
    // enum PressState {
    //     None,
    //     Single,
    //     Double,
    //     Long,
    // }

    pub struct ButtonHandler {
        pub state: ButtonState,
        pub last_pressed_at: fugit::MillisDurationU32,
        pub last_release_at: fugit::MillisDurationU32,
    }

    #[shared]
    struct Shared {
        button_handler: ButtonHandler,
    }

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        button: PB0<Input<PullUp>>,
        delay_handler: DelayMs<TIM2>,
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

        // Configure low frequency clock
        // Initialize Monotonic
        Mono::start(cx.core.SYST, 8_000_000);

        (
            Shared {
                button_handler: ButtonHandler {
                    state: ButtonState::None,
                    last_pressed_at: fugit::MillisDurationU32::from_ticks(0),
                    last_release_at: fugit::MillisDurationU32::from_ticks(0),
                },
            },
            Local {
                led: pc13,
                button,
                delay_handler,
            },
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = EXTI0, priority=1, local = [led, button, delay_handler], shared = [button_handler])]
    fn exti0(cx: exti0::Context) {
        let exti0::LocalResources {
            button,
            delay_handler,
            ..
        } = cx.local;
        let exti0::SharedResources {
            mut button_handler, ..
        } = cx.shared;

        // debounce
        delay_handler.delay_ms(30_u16);
        button.clear_interrupt_pending_bit();
        NVIC::unpend(Interrupt::EXTI0);

        if button.is_low() {
            button_handler.lock(|button_handler| {
                if button_handler.state == ButtonState::None {
                    button_handler.state = ButtonState::Pressed;
                } else if button_handler.state == ButtonState::ReleasedSinglePending {
                    button_handler.state = ButtonState::PressedDoublePending;
                }

                button_handler.last_pressed_at =
                    Mono::now().duration_since_epoch().convert().into();
            });
        } else {
            button_handler.lock(|button_handler| {
                let now: MillisDurationU32 = Mono::now().duration_since_epoch().convert().into();
                let press_duration: MillisDurationU32 = now - button_handler.last_pressed_at;

                if button_handler.state == ButtonState::PressedDoublePending {
                    info!("Double press");
                    button_handler.state = ButtonState::None;
                } else if press_duration.to_millis() >= 1000 {
                    info!("Long press");
                    button_handler.state = ButtonState::None;
                } else if button_handler.state == ButtonState::Pressed {
                    button_handler.state = ButtonState::ReleasedSinglePending;
                }

                button_handler.last_release_at =
                    Mono::now().duration_since_epoch().convert().into();
            });
        }
    }

    impl Format for ButtonState {
        fn format(&self, f: defmt::Formatter<'_>) {
            match self {
                ButtonState::Pressed => defmt::write!(f, "Pressed"),
                _ => defmt::write!(f, "Released"),
            }
        }
    }
}
