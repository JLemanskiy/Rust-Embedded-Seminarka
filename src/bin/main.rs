#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![deny(unsafe_code)]

use seminarka as _;

#[rtic::app(
    device = stm32l4xx_hal::pac,
    dispatchers = [TIM4]
)]
mod app {
    use rtic_monotonics::systick::Systick;
    use rtic_sync::{channel::*, make_channel};
    use stm32l4xx_hal::{
        gpio::{ErasedPin, Output, PushPull},
        prelude::*,
        timer::{Timer,Event}, device::{tim2, TIM2},
    };

    #[shared]
    struct Shared {
        led: ErasedPin<Output<PushPull>>,
        app_timer:Timer<TIM2>,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        defmt::info!("init");
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let mut pwr = cx.device.PWR.constrain(&mut rcc.apb1r1);
        let clocks = rcc.cfgr.freeze(&mut flash.acr, &mut pwr);
        let mut app_timer = Timer::tim2(cx.device.TIM2, 1.Hz(), clocks, &mut rcc.apb1r1);
        let rtic_token = rtic_monotonics::create_systick_token!();
        rtic_monotonics::systick::Systick::start(cx.core.SYST, 8_000_000, rtic_token);
        let mut gpioa = cx.device.GPIOA.split(&mut rcc.ahb2);
        let led = gpioa
            .pa5
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        let mut led1 = led.erase();
        let (s, r) = make_channel!(u8, 5);
        app_timer.listen(Event::TimeOut);
        (Shared { led: led1,app_timer }, Local {})
    }

    #[idle(shared=[])]
    fn idle(_: idle::Context) -> ! {
        loop {
            blink_h::spawn().ok();
            blink_l::spawn().ok();
            continue;
        }
    }

    #[task(priority = 1,shared=[led])]
    async fn blink_h(cx: blink_h::Context) {
        Systick::delay(1000.millis()).await;
        defmt::println!("Led high!");
        let mut led = cx.shared.led;
        led.lock(|led| led.set_high());

    }
    #[task(priority = 1,shared=[led])]
    async fn blink_l(cx: blink_l::Context) {
        Systick::delay(5000.millis()).await;
        let mut led=cx.shared.led;
        defmt::println!("Led low!");
        led.lock(|led| led.set_low());
    }
    #[task(binds=TIM2,shared=[app_timer])]
    fn timer2_timeout(mut cx:timer2_timeout::Context){
        defmt::println!("Hardware interrupt!");
        cx.shared.app_timer.lock(|app_timer| app_timer.clear_interrupt(Event::TimeOut))
    }
}
