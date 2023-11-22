#![no_std]
#![no_main]

use core::{cell::Cell, panic::PanicInfo};
use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
};
use cortex_m_rt::entry; 
use defmt_rtt as _;
use stm32_hal2::{
    self,
    clocks::Clocks,
    gpio::{self, Edge, Pin, PinMode, Port},
    pac::{self, interrupt},
    timer::{OutputCompare, TimChannel, Timer, TimerConfig},
};
static PWM_MODE: Mutex<Cell<u8>> = Mutex::new(Cell::new(0));
#[entry]

fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    // this line is required if you want to take advantage of ST-Link
    stm32_hal2::debug_workaround();

    defmt::println!("Hello, world!");

    let clock_cfg = Clocks::default();
    clock_cfg.setup().unwrap();
    let mut pwm_timer = Timer::new_tim2(
        dp.TIM2,
        2_400.,
        TimerConfig {
            auto_reload_preload: true,
            ..Default::default()
        },
        &clock_cfg,
    );
    pwm_timer.enable_pwm_output(TimChannel::C1, OutputCompare::Pwm1, 0.5);
    pwm_timer.enable();
    // Setup a delay, based on the Cortex-m systick.
    let led = Pin::new(Port::A, 5, PinMode::Alt(1));
    let mut button = Pin::new(Port::C, 13, PinMode::Input);
    button.enable_interrupt(Edge::Falling);
    unsafe {
        NVIC::unmask(pac::Interrupt::EXTI15_10);
    }
    loop {
        defmt::println!("Impreza");
        match free(|cs| PWM_MODE.borrow(cs).get()) {
            0 => pwm_timer.set_duty(TimChannel::C1, 20),
            1 => pwm_timer.set_duty(TimChannel::C1, 50),
            2 => pwm_timer.set_duty(TimChannel::C1, 100),
            2_u8..=u8::MAX => pwm_timer.set_duty(TimChannel::C1, 0),
        }
    }
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    cortex_m::asm::udf()
}

pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
#[interrupt]
fn EXTI15_10() {
    gpio::clear_exti_interrupt(0);
    if free(|cs| PWM_MODE.borrow(cs).get() == 0) {
        free(|cs| PWM_MODE.borrow(cs).set(PWM_MODE.borrow(cs).get() + 1));
    } else {
        free(|cs| PWM_MODE.borrow(cs).set(0));
    }
}
