use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};

use esp_idf_svc::{
    eventloop::{EspEventLoop, System},
    hal::modem::Modem,
    http::{self, server::EspHttpServer},
    io::Read,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

use crate::{rgb::RGBRequest, rgbcontrol::RgbControl};

pub struct Server<'a> {
    httpserver: EspHttpServer<'a>,
    wifi_driver: EspWifi<'a>,
}

impl Server<'_> {
    pub fn new(sys_loop: EspEventLoop<System>, modem: Modem) -> anyhow::Result<Self> {
        let esp_wifi = EspWifi::new(modem, sys_loop.clone(), None)?;
        Ok(Server {
            httpserver: EspHttpServer::new(&http::server::Configuration::default())?,
            wifi_driver: esp_wifi,
        })
    }

    pub fn connect(
        &mut self,
        sys_loop: EspEventLoop<System>,
        configuration: ClientConfiguration,
    ) -> anyhow::Result<()> {
        println!("Connecting to: {}", configuration.ssid);
        let mut wifi = BlockingWifi::wrap(&mut self.wifi_driver, sys_loop)?;
        wifi.set_configuration(&Configuration::Client(configuration))?;
        wifi.start()?;
        wifi.connect()?;
        wifi.wait_netif_up()?;
        let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

        log::info!("Connection establised: {}", ip_info.ip);

        Ok(())
    }

    pub fn get_ip_addr(&self) -> Ipv4Addr {
        if let Ok(info) = self.wifi_driver.sta_netif().get_ip_info() {
            info.ip
        } else {
            Ipv4Addr::new(0, 0, 0, 0)
        }
    }

    pub fn handle_response(
        &mut self,
        rgb_controller: Arc<Mutex<RgbControl>>,
    ) -> anyhow::Result<()> {
        let rgb_led_color_clone = rgb_controller.clone();
        self.httpserver
            .fn_handler("/get", http::Method::Get, move |request| {
                let rgb_led_color_lock = rgb_led_color_clone.lock().unwrap();
                let mut response = request.into_ok_response()?;
                response.write(&rgb_led_color_lock.get_color().to_u32().to_le_bytes())?;
                drop(rgb_led_color_lock);
                Ok::<(), anyhow::Error>(())
            })?;

        let rgb_led_color_clone = rgb_controller.clone();
        self.httpserver
            .fn_handler("/set", http::Method::Post, move |mut request| {
                let mut buf = [0 as u8; 3];
                request.read_exact(&mut buf)?;
                if let Ok(packet) = RGBRequest::new(buf) {
                    let mut rgb_lock = rgb_led_color_clone.lock().unwrap();
                    rgb_lock.set_color(packet.color)?;
                }
                Ok::<(), anyhow::Error>(())
            })?;
        Ok(())
    }
}
