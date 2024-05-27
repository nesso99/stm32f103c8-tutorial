#![no_std]
#![no_main]

use core::{
    cell::RefCell,
    sync::atomic::{AtomicU32, Ordering},
};

use cortex_m::{
    asm::nop,
    interrupt::{self as cinterrupt, Mutex},
};
use cortex_m_rt::entry;
use defmt::info;
use stm32f1xx_hal::{
    gpio::{Output, PushPull, PC13},
    pac::{self, interrupt, Peripherals, TIM1},
    prelude::*,
    timer::{CounterMs, Event},
};
use {defmt_rtt as _, panic_probe as _};

type Pc13Pin = PC13<Output<PushPull>>;

static G_PC13: Mutex<RefCell<Option<Pc13Pin>>> = Mutex::new(RefCell::new(None));
static G_TIM1: Mutex<RefCell<Option<CounterMs<TIM1>>>> = Mutex::new(RefCell::new(None));
static G_FREQ: AtomicU32 = AtomicU32::new(1000);

#[entry]
fn main() -> ! {
    info!("irq_blinky example");
    let dp = Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut gpioc = dp.GPIOC.split();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(8.MHz())
        .pclk1(8.MHz())
        .freeze(&mut flash.acr);
    let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    pc13.set_high();

    let mut timer = dp.TIM1.counter_ms(&clocks);
    let val = G_FREQ.load(Ordering::Relaxed);
    timer.start(val.millis()).unwrap();

    // Generate an interrupt when the timer expires
    timer.listen(Event::Update);

    cinterrupt::free(|cs| {
        G_PC13.borrow(cs).replace(Some(pc13));
        G_TIM1.borrow(cs).replace(Some(timer));
    });

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::TIM1_UP);
    }

    loop {
        // I tried wfi and wfe but it throw error when flash
        nop();
    }
}

#[interrupt]
fn TIM1_UP() {
    let val = G_FREQ.load(Ordering::Relaxed);
    info!("currenv val: {:?}", val);
    G_FREQ.store(val.saturating_add(100), Ordering::Relaxed);

    cinterrupt::free(|cs| {
        let mut tim1 = G_TIM1.borrow(cs).borrow_mut();
        let mut pc13 = G_PC13.borrow(cs).borrow_mut();
        let tim1 = tim1.as_mut().unwrap();

        pc13.as_mut().unwrap().toggle();
        tim1.start(val.millis()).unwrap();
        // tim1.wait().unwrap();
    });
}
