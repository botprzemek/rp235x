pub struct Net;

impl Net {
    pub async fn init_stack(
        seed: [u8; 8],
        net_device: NetDriver<'static>,
        control: &mut Control<'static>,
    ) -> (
        Stack<'static>,
        embassy_net::Runner<'static, NetDriver<'static>>,
    ) {
        let clm_ptr = core::hint::black_box(CLM_ADDR as *const u8);
        let clm_slice: &[u8] = unsafe { core::slice::from_raw_parts(clm_ptr, CLM_LEN) };
        let clm_aligned: &cyw43::Aligned<cyw43::A4, [u8]> =
            unsafe { core::mem::transmute(clm_slice) };
        let dhcp_config = embassy_net::DhcpConfig::default();
        let net_config = embassy_net::Config::dhcpv4(dhcp_config);

        control.init(clm_aligned).await;
        control
            .set_power_management(cyw43::PowerManagementMode::None)
            .await;

        let sockets = SOCKETS.init(embassy_net::StackResources::<5>::new());

        let seed_u64 = u64::from_le_bytes(seed);

        embassy_net::new(net_device, net_config, sockets, seed_u64)
    }

    pub async fn init_wifi(control: &mut Control<'static>) {
        let config = Config::read().unwrap();

        while let Err(err) = control
            .join(
                config.get_wifi_ssid().unwrap(),
                JoinOptions::new(config.get_wifi_password().unwrap().as_bytes()),
            )
            .await
        {
            error!("join failed: {:?}", err);
            embassy_time::Timer::after_secs(1).await;
        }
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    info!("net::connection up");
    runner.run().await
}
