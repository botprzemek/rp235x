//! Embassy network stack and Wi-Fi association initialization.

use core::hint;
use core::mem;

use config::NetworkConfig;
use cyw43::{A4, Aligned, Control, JoinOptions, NetDriver, PowerManagementMode};
use embassy_net::{Config as NetConfig, DhcpConfig, Runner, Stack, StackResources};
use embassy_time::Timer;
use static_cell::StaticCell;

/// Initialized Embassy network [`Stack`] and its background [`Runner`].
pub struct InitializedNetStack {
    pub stack: Stack<'static>,
    pub runner: Runner<'static, NetDriver<'static>>,
}

/// Network stack and Wi-Fi connection helper.
pub struct Net;

static SOCKETS: StaticCell<StackResources<5>> = StaticCell::new();

const CLM_MASK: usize = 0x5a5a_5a5a;
const CLM_ADDR: usize = 0x1024_0000 ^ CLM_MASK;
const CLM_LEN: usize = 984;

impl Net {
    /// Initializes the CLM blob on the CYW43 controller and creates the DHCPv4 network stack.
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

    /// Joins the configured Wi-Fi network, retrying until association succeeds.
    ///
    /// # Panics
    ///
    /// Panics if `config` contains invalid or empty Wi-Fi credentials.
    pub async fn init_wifi(control: &mut Control<'static>, config: &impl NetworkConfig) {
        let ssid = config.wifi_ssid();
        let password = config.wifi_password();

        let (ssid, password_bytes) = match (ssid, password) {
            (Ok(ssid_str), Ok(password_str))
                if !ssid_str.is_empty() && !password_str.is_empty() =>
            {
                (ssid_str, password_str.as_bytes())
            }
            _ => {
                panic!("Invalid or empty Wi-Fi credentials.");
            }
        };

        while control
            .join(ssid, JoinOptions::new(password_bytes))
            .await
            .is_err()
        {
            Timer::after_secs(1).await;
        }
    }

    #[inline(always)]
    fn load_clm() -> &'static Aligned<A4, [u8; CLM_LEN]> {
        let address = CLM_ADDR ^ CLM_MASK;

        assert!(
            address.is_multiple_of(mem::align_of::<Aligned<A4, [u8; CLM_LEN]>>()),
            "Memory alignment violation detected at hardware boundary for CYW43 CLM at {address:#x}"
        );

        let pointer = hint::black_box(address as *const Aligned<A4, [u8; CLM_LEN]>);
        // SAFETY: `address` is 4-byte aligned (checked above) and points to the CYW43 CLM blob
        // flashed at `0x1024_0000` with length `CLM_LEN`.
        // Justification: Zero-copy access to memory-mapped XIP flash on RP235x.
        unsafe { &*pointer }
    }
}

/// Background Embassy task driving the TCP/IP and UDP network stack.
#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, NetDriver<'static>>) -> ! {
    runner.run().await
}
