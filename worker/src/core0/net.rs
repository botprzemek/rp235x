use core::hint;
use core::mem;

use config::NetworkConfig;
use cyw43::{A4, Aligned, Control, JoinOptions, NetDriver, PowerManagementMode};
use embassy_net::{Config as NetConfig, DhcpConfig, Runner, Stack, StackResources};
use embassy_time::Timer;
use static_cell::StaticCell;

pub struct InitializedNetStack {
    pub stack: Stack<'static>,
    pub runner: Runner<'static, NetDriver<'static>>,
}

pub struct Net;

static SOCKETS: StaticCell<StackResources<5>> = StaticCell::new();

const CLM_MASK: usize = 0x5a5a_5a5a;
const CLM_ADDR: usize = 0x1024_0000 ^ CLM_MASK;
const CLM_LEN: usize = 984;

impl Net {

    pub async fn init_stack(
        control: &mut Control<'static>,
        net_driver: NetDriver<'static>,
        seed: [u8; 8],
    ) -> InitializedNetStack {
        let clm = Self::load_clm().as_slice();
        control.init(clm).await;
        control
            .set_power_management(PowerManagementMode::None)
            .await;

        let config = {
            let dhcp_config = DhcpConfig::default();
            NetConfig::dhcpv4(dhcp_config)
        };
        let resources = SOCKETS.init(StackResources::<5>::new());
        let random_seed = u64::from_le_bytes(seed);

        let (stack, runner) = embassy_net::new(net_driver, config, resources, random_seed);
        InitializedNetStack { stack, runner }
    }

    pub async fn try_join_wifi(
        control: &mut Control<'static>,
        config: &impl NetworkConfig,
        max_retries: usize,
    ) -> bool {
        let ssid = config.wifi_ssid();
        let password = config.wifi_password();

        let (ssid, password_bytes) = match (ssid, password) {
            (Ok(ssid_str), Ok(password_str))
                if !ssid_str.is_empty() && !password_str.is_empty() =>
            {
                (ssid_str, password_str.as_bytes())
            }
            _ => return false,
        };

        for attempt in 1..=max_retries {
            defmt::info!(
                "Connecting to Wi-Fi '{}' (attempt {}/{})",
                ssid,
                attempt,
                max_retries
            );
            if control
                .join(ssid, JoinOptions::new(password_bytes))
                .await
                .is_ok()
            {
                defmt::info!("Wi-Fi association successful!");
                return true;
            }
            Timer::after_secs(1).await;
        }

        defmt::warn!(
            "Failed to associate with Wi-Fi after {} attempts",
            max_retries
        );
        false
    }

    #[inline(always)]
    fn load_clm() -> &'static Aligned<A4, [u8; CLM_LEN]> {
        let address = CLM_ADDR ^ CLM_MASK;

        assert!(
            address.is_multiple_of(mem::align_of::<Aligned<A4, [u8; CLM_LEN]>>()),
            "Memory alignment violation detected at hardware boundary for CYW43 CLM at {address:#x}"
        );

        let pointer = hint::black_box(address as *const Aligned<A4, [u8; CLM_LEN]>);

        unsafe { &*pointer }
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, NetDriver<'static>>) -> ! {
    runner.run().await
}
