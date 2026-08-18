use crate::core0::{
    cyw43::{Cyw43, cyw43_task},
    net::{Net, net_task},
};
use crate::peripherals::{NetPeripherals, TrngPeripherals};
use crate::state::{Input, Machine};
use config::Config;
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
    async fn handle_rest(&mut self) -> ();
}

impl Handler for Machine {
    async fn handle_boot(&mut self) {
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
            Some(_) => self.transition(Input::WifiConnected),
            None => self.transition(Input::WifiFailed),
        }
    }

    async fn handle_rest(&mut self) {
        embassy_time::Timer::after_secs(1).await;
    }
}
