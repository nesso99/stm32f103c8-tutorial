//!
//! If rtc not working, unplug all gpio and try again, good luck
//!

#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true)]
mod app {
    use stm32f1xx_hal::{
        gpio::{Output, PushPull, PC13},
        prelude::*,
        rtc::Rtc,
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        rtc: Rtc,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let dp = cx.device;
        let rcc = dp.RCC.constrain();
        let mut gpioc = dp.GPIOC.split();
        let mut pwr = dp.PWR;

        let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        pc13.set_high();

        let mut backup_domain = rcc.bkp.constrain(dp.BKP, &mut pwr);
        let rtc = Rtc::new(dp.RTC, &mut backup_domain);

        (Shared {}, Local { led: pc13, rtc })
    }

    #[idle(local = [led, rtc])]
    fn idle(cx: idle::Context) -> ! {
        let idle::LocalResources { led, rtc, .. } = cx.local;
        loop {
            rtc.set_time(0);
            rtc.set_alarm(1);
            nb::block!(rtc.wait_alarm()).unwrap();
            led.toggle();
        }
    }
}
