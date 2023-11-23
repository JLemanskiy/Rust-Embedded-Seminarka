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
        timer::{Timer,Event},
    };

    #[shared]
    struct Shared {
        led: ErasedPin<Output<PushPull>>,
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
        let mut app_timer = Timer::tim2(cx.device.TIM2, 1.kHz(), clocks, &mut rcc.apb1r1);
        let rtic_token = rtic_monotonics::create_systick_token!();
        rtic_monotonics::systick::Systick::start(cx.core.SYST, 8_000_000, rtic_token);
        let mut gpioa = cx.device.GPIOA.split(&mut rcc.ahb2);
        let led = gpioa
            .pa5
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        let mut led1 = led.erase();
        let (s, r) = make_channel!(u8, 5);
        count_blinks::spawn(r).ok();
        blink::spawn(s).ok();
        app_timer.listen(Event::TimeOut);
        (Shared { led: led1 }, Local {})
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        defmt::info!("dupa");
        loop {
            continue;
        }
    }

    #[task(priority = 1,shared=[led])]
    async fn blink(cx: blink::Context, mut sender: Sender<'static, u8, 5>) {
        defmt::info!("Slow blink");
        Systick::delay(50.millis()).await;
        let mut led = cx.shared.led;
        led.lock(|led| led.set_high());
        Systick::delay(50.millis()).await;
        led.lock(|led| led.set_low());
        sender.send(1).await.expect("Sending failed!")
    }
    #[task(priority = 1)]
    async fn count_blinks(cx: count_blinks::Context, mut reciever: Receiver<'static, u8, 5>) {
        let mut counter: u8 = 0;
        while let Ok(value) = reciever.recv().await {
            counter += value
        }
        if counter == 100 {
            defmt::println!("{}", counter);
            counter = 0;
        }
    }
    #[task(binds=TIM2,shared=[])]
    fn timer2_timeout(cx:timer2_timeout::Context){
        defmt::println!("Hardware interrupt!");
    }
}
