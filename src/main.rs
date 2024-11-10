use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::ledc::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::sys::esp_random;

fn rand_f32() -> f32 {
    let rng;
    unsafe { rng = esp_random() as f32 / u32::MAX as f32; }
    rng
}

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();

    println!("Configuring output channel");
    let timer_config = config::TimerConfig::new().frequency(25.kHz().into());
    let peripherals = Peripherals::take()?;
    let ledc_timer_driver_b = LedcTimerDriver::new(peripherals.ledc.timer0, &timer_config)?;
    let ledc_timer_driver_r = LedcTimerDriver::new(peripherals.ledc.timer1, &timer_config)?;
    let ledc_timer_driver_g = LedcTimerDriver::new(peripherals.ledc.timer2, &timer_config)?;


    let mut channel_b = LedcDriver::new(
        peripherals.ledc.channel0,
        ledc_timer_driver_b,
        peripherals.pins.gpio2,
    )?;
    
    let mut channel_g = LedcDriver::new(
        peripherals.ledc.channel1,
        ledc_timer_driver_r,
        peripherals.pins.gpio3,
    )?;

    let mut channel_r = LedcDriver::new(
        peripherals.ledc.channel2,
        ledc_timer_driver_g,
        peripherals.pins.gpio10
    )?;
    println!("Starting duty-cycle loop");

    let max_duty = channel_b.get_max_duty();

    loop {
        FreeRtos::delay_ms(1000);
        channel_r.set_duty((rand_f32() * max_duty as f32) as u32)?;
        channel_g.set_duty((rand_f32() * max_duty as f32) as u32)?;
        channel_b.set_duty((rand_f32() * max_duty as f32) as u32)?;
    }
}
