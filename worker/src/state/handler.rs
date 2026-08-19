use crate::core0::{
    cyw43::{Cyw43, cyw43_task},
    net::{Net, net_task},
};
use crate::peripherals::{NetPeripherals, TrngPeripherals};
use crate::state::{Input, Machine};
use config::{Config, print::Print};
use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub static CORE1_READY_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

pub trait Handler {
    async fn handle_boot(&mut self) -> ();
    async fn handle_sync(&mut self) -> ();
    async fn handle_networking(
        &mut self,
        spawner: Spawner,
        net: NetPeripherals,
        trng: TrngPeripherals,
    ) -> ();
    async fn handle_running(&mut self) -> ();
    async fn handle_error_recovery(&mut self) -> ();
}

impl Handler for Machine {
    async fn handle_boot(&mut self) {
        let config = match Config::read() {
            Ok(config) => config,
            Err(_) => return self.transition(Input::BootFailed),
        };

        if config.print().is_err() {
            return self.transition(Input::BootFailed);
        }

        self.transition(Input::BootSuccess);
    }

    async fn handle_sync(&mut self) {
        CORE1_READY_SIGNAL.wait().await;

        self.transition(Input::CoreSynced);
    }

    async fn handle_networking(
        &mut self,
        spawner: Spawner,
        net: NetPeripherals,
        trng: TrngPeripherals,
    ) {
        let seed = TrngPeripherals::init(trng).await;
        let (net_device, mut control, runner) = Cyw43::init(net).await;
        spawner.spawn(unwrap!(cyw43_task(runner)));

        let (stack, runner) = Net::init_stack(&mut control, net_device, seed).await;
        spawner.spawn(unwrap!(net_task(runner)));

        let config = Config::read().unwrap();
        Net::init_wifi(&mut control, config).await;
        stack.wait_link_up().await;
        stack.wait_config_up().await;

        match stack.config_v4() {
            Some(config_v4) => {
                defmt::info!(
                    "NetworkConfig::Address               {}",
                    &config_v4.address
                );
                defmt::info!(
                    "NetworkConfig::Gateway               {}",
                    &config_v4.gateway.unwrap()
                );

                self.transition(Input::WifiConnected);
            }
            None => self.transition(Input::WifiFailed),
        }
    }

    async fn handle_running(&mut self) {
        embassy_time::Timer::after_secs(1).await;
    }

    async fn handle_error_recovery(&mut self) {
        embassy_time::Timer::after_secs(1).await;
    }
}
