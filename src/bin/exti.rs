#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m::interrupt::{self as cinterrupt, Mutex};
use cortex_m_rt::entry;
use defmt::info;
use stm32f1xx_hal::{
    gpio::{Edge, ExtiPin, GpioExt, Input, Output, PullUp, PushPull, PB12, PB13, PC13},
    pac::{self, interrupt, Peripherals},
    prelude::*,
};
use {defmt_rtt as _, panic_probe as _};

type Pc13Pin = PC13<Output<PushPull>>;
type Pb12Pin = PB12<Input<PullUp>>;
type Pb13Pin = PB13<Input<PullUp>>;

static G_PC13: Mutex<RefCell<Option<Pc13Pin>>> = Mutex::new(RefCell::new(None));
static G_PB12: Mutex<RefCell<Option<Pb12Pin>>> = Mutex::new(RefCell::new(None));
static G_PB13: Mutex<RefCell<Option<Pb13Pin>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut dp = Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();
    let mut gpioc = dp.GPIOC.split();
    let mut gpiob = dp.GPIOB.split();

    let clocks = rcc.cfgr.use_hse(8.MHz()).freeze(&mut flash.acr);
    let mut delay = dp.TIM2.delay_us(&clocks);
    let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    pc13.set_high();

    let mut pb12 = gpiob.pb12.into_pull_up_input(&mut gpiob.crh);
    pb12.make_interrupt_source(&mut afio);
    pb12.trigger_on_edge(&mut dp.EXTI, Edge::Falling);
    pb12.enable_interrupt(&mut dp.EXTI);

    let mut pb13 = gpiob.pb13.into_pull_up_input(&mut gpiob.crh);
    pb13.make_interrupt_source(&mut afio);
    pb13.trigger_on_edge(&mut dp.EXTI, Edge::Falling);
    pb13.enable_interrupt(&mut dp.EXTI);

    cinterrupt::free(|cs| {
        G_PC13.borrow(cs).replace(Some(pc13));
        G_PB12.borrow(cs).replace(Some(pb12));
        G_PB13.borrow(cs).replace(Some(pb13));
    });

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::EXTI15_10);
    }

    loop {
        info!("toggle");
        delay.delay(10.secs());
    }
}

#[interrupt]
fn EXTI15_10() {
    // cannot identify button on same exti, I tried on c versio

    cinterrupt::free(|cs| {
        // if let Some(ref mut pc13) = G_PC13.borrow(cs).borrow_mut().as_mut() {
        //     pc13.toggle();
        // }
        let mut pb12 = G_PB12.borrow(cs).borrow_mut();
        let mut pb13 = G_PB13.borrow(cs).borrow_mut();

        if pb12.as_ref().unwrap().check_interrupt() {
            info!("pb12");
            pb12.as_mut().unwrap().clear_interrupt_pending_bit();
        }

        if pb13.as_ref().unwrap().check_interrupt() {
            info!("pb13");
            pb13.as_mut().unwrap().clear_interrupt_pending_bit();
        }
    });
}
