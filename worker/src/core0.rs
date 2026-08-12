use defmt::{info, unwrap};
use embassy_executor::Executor;
use static_cell::StaticCell;

use crate::net;

pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

pub struct Core0;

impl Core0 {
    pub fn entry(
        net_peripherals: net::NetPeripherals,
        trng_peripherals: net::TrngPeripherals,
    ) -> ! {
        let executor = Executor::new();

        EXECUTOR.init(executor).run(|spawner| {
            spawner.spawn(unwrap!(task(spawner, net_peripherals, trng_peripherals)));
        })
    }
}

#[embassy_executor::task]
async fn task(
    spawner: embassy_executor::Spawner,
    net_peripherals: net::NetPeripherals,
    trng_peripherals: net::TrngPeripherals,
) {
    let seed = net::Net::init_trng(trng_peripherals).await;
    let (net_device, mut control, runner) = net::Net::init_cyw43(net_peripherals).await;

    spawner.spawn(unwrap!(net::cyw43_task(runner)));

    let (stack, runner) = net::Net::init_stack(seed, net_device, &mut control).await;

    spawner.spawn(unwrap!(net::net_task(runner)));

    net::Net::init_wifi(&mut control).await;
    stack.wait_link_up().await;
    stack.wait_config_up().await;

    if let Some(config) = stack.config_v4() {
        info!("net::stack up");
        info!("   ::ip      {}", config.address.address());

        if let Some(gateway) = config.gateway {
            info!("   ::gateway {}", gateway);
        }
    }

    spawner.spawn(unwrap!(net::ntp_task(stack)));
    spawner.spawn(unwrap!(net::game_task(stack)));
}
