
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::ledc::config::TimerConfig;
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver};
use esp_idf_hal::sys::esp_random;
use esp_idf_hal::uart::{self, UartDriver};
use esp_idf_hal::{delay, gpio, prelude::*};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use std::sync::{Arc, Mutex};

pub mod credentials;
pub mod rgb;
pub mod server;

fn rand_f32() -> f32 {
    let rng;
    unsafe {
        rng = esp_random() as f32 / u32::MAX as f32;
    }
    rng
}

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();
    let sys_loop = EspSystemEventLoop::take()?;

    let timer_config = TimerConfig::new().frequency(25.kHz().into());
    let peripherals = Peripherals::take()?;
    //let mut server = server::Server::new(sys_loop.clone(), peripherals.modem)?;
    let color = Arc::new(Mutex::new(rgb::RGBLedColor::new(255, 255, 255)));

    //server.connect(sys_loop.clone())?;
    //server.handle_response(color.clone())?;

    let tx = peripherals.pins.gpio20;
    let rx = peripherals.pins.gpio21;

    println!("Initializing UART...");
    let config = uart::config::Config::new().baudrate(Hertz(115_200));
    let uart = UartDriver::new(
        peripherals.uart0,
        tx,
        rx,
        Option::<gpio::Gpio0>::None,
        Option::<gpio::Gpio1>::None,
        &config,
    )?;
    println!("UART INITIALIZED!");

    // leds
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
        peripherals.pins.gpio10,
    )?;

    let color_clone = color.clone();
    loop {
        FreeRtos::delay_ms(100);
        //let color = color_clone.lock().unwrap();
        //channel_r.set_duty(color.red as u32)?;
        //channel_g.set_duty(color.green as u32)?;
        //channel_b.set_duty(color.blue as u32)?;
        uart.write("pidor jopa".as_bytes())?;

        let mut buf = [0_u8; 64];
        match uart.read(&mut buf, delay::NON_BLOCK) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    println!("got {} bytes: {:#?}", bytes_read, &buf[..bytes_read]);
                } else {
                    println!("no buffer");
                }
            }
            Err(e) => {
                println!("read error: {:?}", e);
            }
        }
    }
}
