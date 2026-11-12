#[cfg(feature = "rp235x")]
use crate::state::{Input, Machine};

#[cfg(feature = "rp235x")]
use crate::core0::cyw43::{Cyw43, cyw43_task};
#[cfg(feature = "rp235x")]
use crate::core0::dhcp::run_dhcp_server;
#[cfg(feature = "rp235x")]
use crate::core0::net::{Net, net_task};
#[cfg(feature = "rp235x")]
use crate::core0::udp_handler::unified_udp_task;
#[cfg(feature = "rp235x")]
use crate::peripherals::{NetPeripherals, TrngPeripherals};
#[cfg(feature = "rp235x")]
use config::Config;
#[cfg(feature = "rp235x")]
use config::flash::FlashReader;
#[cfg(feature = "rp235x")]
use config::print::Print;
#[cfg(feature = "rp235x")]
use defmt::Format;
#[cfg(feature = "rp235x")]
use embassy_executor::Spawner;
#[cfg(feature = "rp235x")]
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StaticConfigV4};
#[cfg(feature = "rp235x")]
use embassy_rp::Peri;
#[cfg(feature = "rp235x")]
use embassy_rp::peripherals::{FLASH, WATCHDOG};
#[cfg(feature = "rp235x")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "rp235x")]
use embassy_sync::signal::Signal;
#[cfg(feature = "rp235x")]
use embassy_time::Timer;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "rp235x", derive(Format))]
pub enum StateHandler {
    #[default]
    Boot = 0x01,
    Sync = 0x02,
    Networking = 0x03,
    Running = 0x04,
    ErrorRecovery = 0x05,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "rp235x", derive(Format))]
pub struct BootHandler;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "rp235x", derive(Format))]
pub struct SyncHandler;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "rp235x", derive(Format))]
pub struct NetworkingHandler;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "rp235x", derive(Format))]
pub struct RunningHandler;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "rp235x", derive(Format))]
pub struct ErrorRecoveryHandler;

#[cfg(feature = "rp235x")]
pub static CORE1_READY_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[cfg(not(feature = "rp235x"))]
pub struct LinuxReadySignal {
    signaled: std::sync::atomic::AtomicBool,
}

#[cfg(not(feature = "rp235x"))]
impl Default for LinuxReadySignal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "rp235x"))]
impl LinuxReadySignal {
    pub const fn new() -> Self {
        Self {
            signaled: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn signal(&self) {
        self.signaled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn wait(&self) {
        while !self.signaled.load(std::sync::atomic::Ordering::SeqCst) {
            std::thread::yield_now();
        }
    }
}

#[cfg(not(feature = "rp235x"))]
pub static CORE1_READY_SIGNAL: LinuxReadySignal = LinuxReadySignal::new();

#[cfg(feature = "rp235x")]
impl Machine {

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

    pub async fn handle_sync(&mut self) {
        CORE1_READY_SIGNAL.wait().await;

        self.transition(Input::CoreSynced);
    }

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

            let ap_ip = Ipv4Address::new(192, 168, 4, 1);
            let subnet = Ipv4Cidr::new(ap_ip, 24);
            initialized_net
                .stack
                .set_config_v4(embassy_net::ConfigV4::Static(StaticConfigV4 {
                    address: subnet,
                    gateway: Some(ap_ip),
                    dns_servers: heapless::Vec::new(),
                }));

            cyw43_driver.control.start_ap_open("Worker-Setup", 1).await;
            defmt::info!("AP 'Worker-Setup' started! Listening on UDP port 9000...");

            spawner.spawn(defmt::unwrap!(dhcp_task(initialized_net.stack)));

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

#[cfg(feature = "linux")]
impl Machine {

    pub async fn handle_boot(&mut self) {
        println!("🔧 [Core 0 Thread] [BOOT] Initializing Linux worker state...");
        self.transition(Input::BootSuccess);
    }

    pub async fn handle_sync(&mut self) {
        println!("⏳ [Core 0 Thread] [SYNC] Waiting for Core 1 ready signal...");
        CORE1_READY_SIGNAL.wait();
        println!("✅ [Core 0 Thread] [SYNC] Core 1 is ready! Handshake complete.");
        self.transition(Input::CoreSynced);
    }

    pub async fn handle_networking(&mut self) {
        println!("🌐 [Core 0 Thread] [NETWORKING] Initializing network stack...");
        self.transition(Input::WifiConnected);
        println!("🚀 [Core 0 Thread] [RUNNING] Starting UDP worker loop on port 9000...");
        if let Err(e) = crate::core0::entry::run_udp_worker().await {
            eprintln!("⚠️ [Core 0 Thread] Networking error: {e}");
            self.transition(Input::ErrorOccurred);
        }
    }

    #[allow(dead_code)]
    pub async fn handle_running(&mut self) {}

    #[allow(dead_code)]
    pub async fn handle_error_recovery(&mut self) {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

#[cfg(feature = "rp235x")]
#[embassy_executor::task]
pub async fn dhcp_task(stack: Stack<'static>) -> ! {
    run_dhcp_server(stack).await
}
