//! Inherent state handlers on [`Machine`] and the UDP client task.

use config::Config;
use config::flash::FlashReader;
use config::print::Print;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StaticConfigV4};
use embassy_rp::Peri;
use embassy_rp::peripherals::{FLASH, WATCHDOG};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;

use crate::core0::cyw43::{Cyw43, cyw43_task};
use crate::core0::dhcp::run_dhcp_server;
use crate::core0::net::{Net, net_task};
use crate::core0::udp_handler::unified_udp_task;
use crate::peripherals::{NetPeripherals, TrngPeripherals};
use crate::state::{Input, Machine};

/// Signal emitted by Core 1 once its HUB75 display task has been spawned.
pub static CORE1_READY_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

impl Machine {
    /// Verifies and prints the flash configuration during boot.
    pub async fn handle_boot(&mut self) {
        let Ok(config) = Config::read() else {
            self.transition(Input::BootFailed);
            return;
        };

        if config.print().is_err() {
            self.transition(Input::BootFailed);
            return;
        }

        self.transition(Input::BootSuccess);
    }

    /// Waits for Core 1 to signal readiness.
    pub async fn handle_sync(&mut self) {
        CORE1_READY_SIGNAL.wait().await;

        self.transition(Input::CoreSynced);
    }

    /// Initializes the TRNG, CYW43 Wi-Fi chip, and manages STA connection or AP setup mode.
    pub async fn handle_networking(
        &mut self,
        spawner: Spawner,
        net: NetPeripherals,
        trng: TrngPeripherals,
        watchdog: Peri<'static, WATCHDOG>,
        flash: Peri<'static, FLASH>,
    ) {
        let seed = TrngPeripherals::init(trng).await;
        let mut cyw43_driver = Cyw43::init(net).await;
        spawner.spawn(defmt::unwrap!(cyw43_task(cyw43_driver.runner)));

        let initialized_net =
            Net::init_stack(&mut cyw43_driver.control, cyw43_driver.net_driver, seed).await;
        spawner.spawn(defmt::unwrap!(net_task(initialized_net.runner)));

        let loaded_config = Config::read().ok();

        let mut connected_sta = false;
        if let Some(config) = loaded_config {
            defmt::info!("Attempting connection with existing config...");
            if Net::try_join_wifi(&mut cyw43_driver.control, config, 3).await {
                initialized_net.stack.wait_link_up().await;
                // Wait for DHCP lease with timeout
                let mut dhcp_ok = false;
                for _ in 0..10 {
                    if initialized_net.stack.is_config_up() {
                        dhcp_ok = true;
                        break;
                    }
                    Timer::after_millis(500).await;
                }

                if !dhcp_ok {
                    return;
                }

                let config_v4 = match initialized_net.stack.config_v4() {
                    Some(config_v4) => config_v4,
                    None => return,
                };

                defmt::info!(
                    "NetworkConfig::Address               {}",
                    &config_v4.address
                );

                if let Some(ref gateway) = config_v4.gateway {
                    defmt::info!("NetworkConfig::Gateway               {}", gateway);
                }

                self.transition(Input::WifiConnected);
                connected_sta = true;
            }
        }

        if connected_sta {
            spawner.spawn(defmt::unwrap!(unified_udp_task(
                initialized_net.stack,
                watchdog,
                flash,
                false
            )));
        } else {
            defmt::warn!("Wi-Fi connection unavailable. Launching AP Setup Hotspot...");
            self.transition(Input::WifiFailed);

            // Configure Static IP 192.168.4.1/24 for AP mode
            let ap_ip = Ipv4Address::new(192, 168, 4, 1);
            let subnet = Ipv4Cidr::new(ap_ip, 24);
            initialized_net
                .stack
                .set_config_v4(embassy_net::ConfigV4::Static(StaticConfigV4 {
                    address: subnet,
                    gateway: Some(ap_ip),
                    dns_servers: heapless::Vec::new(),
                }));

            // Start CYW43 in AP mode (Open network 'Worker-Setup' on channel 1)
            cyw43_driver.control.start_ap_open("Worker-Setup", 1).await;
            defmt::info!("AP 'Worker-Setup' started! Listening on UDP port 8001...");

            // Spawn DHCP server in background
            spawner.spawn(defmt::unwrap!(dhcp_task(initialized_net.stack)));

            // Run unified UDP task on port 8001 for OTA Wi-Fi configuration
            spawner.spawn(defmt::unwrap!(unified_udp_task(
                initialized_net.stack,
                watchdog,
                flash,
                true
            )));
        }
    }

    #[allow(dead_code)]
    async fn handle_running(&mut self) {}

    #[allow(dead_code)]
    async fn handle_error_recovery(&mut self) {
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
pub async fn dhcp_task(stack: Stack<'static>) -> ! {
    run_dhcp_server(stack).await
}
