use config::NetworkConfig;
use cyw43::{A4, Aligned, Control, JoinOptions, NetDriver, PowerManagementMode};
use embassy_net::{Config as NetConfig, DhcpConfig, Runner, Stack, StackResources, new};
use static_cell::StaticCell;

pub struct Net;

static SOCKETS: StaticCell<StackResources<5>> = StaticCell::new();

const CLM_MASK: usize = 0x5A5A5A5A;
const CLM_ADDR: usize = 0x10240000 ^ CLM_MASK;
const CLM_LEN: usize = 984;

impl Net {
    pub async fn init_stack(
        control: &mut Control<'static>,
        net_driver: NetDriver<'static>,
        seed: [u8; 8],
    ) -> (Stack<'static>, Runner<'static, NetDriver<'static>>) {
        let clm = Self::load_clm().as_slice();
        control.init(clm).await;
        control
            .set_power_management(PowerManagementMode::None)
            .await;

        let config = {
            let config = DhcpConfig::default();
            NetConfig::dhcpv4(config)
        };
        let resources = SOCKETS.init(StackResources::<5>::new());
        let random_seed = u64::from_le_bytes(seed);

        new(net_driver, config, resources, random_seed)
    }

    pub async fn init_wifi(control: &mut Control<'static>, config: &impl NetworkConfig) {
        let ssid = config.get_wifi_ssid();
        let password = config.get_wifi_password();

        let (ssid, pass_bytes) = match (ssid, password) {
            (Ok(s), Ok(p)) if !s.is_empty() && !p.is_empty() => (s, p.as_bytes()),
            _ => {
                panic!("Invalid or empty Wi-Fi credentials.");
            }
        };

        while let Err(_err) = control.join(ssid, JoinOptions::new(pass_bytes)).await {
            embassy_time::Timer::after_secs(1).await;
        }
    }

    #[inline(always)]
    fn load_clm() -> &'static Aligned<A4, [u8; CLM_LEN]> {
        let address = CLM_ADDR ^ CLM_MASK;

        assert!(
            address.is_multiple_of(core::mem::align_of::<Aligned<A4, [u8; CLM_LEN]>>()),
            "Memory alignment violation detected at hardware boundary."
        );

        let ptr = core::hint::black_box(address as *const Aligned<A4, [u8; CLM_LEN]>);
        unsafe { &*ptr }
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}
