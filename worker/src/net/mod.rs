use cyw43::{Control, JoinOptions, NetDriver, SpiBus};
use cyw43_pio::PioSpi;
use defmt::{error, info};
use embassy_net::{
    Stack,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_rp::{
    Peri, dma,
    gpio::{self, Output},
    peripherals,
    pio::Pio,
    trng,
};
use static_cell::StaticCell;

use crate::{interrupts, shared};

static UDP_RX_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
static UDP_TX_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

static WIFI_SSID: &str = env!("WIFI_SSID");
static WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

static WIFI_STATE: StaticCell<cyw43::State> = StaticCell::new();
static SOCKETS: StaticCell<embassy_net::StackResources<5>> = StaticCell::new();

static FW_ADDR: usize = 0x10200000;
static FW_LEN: usize = 231077;

static NVRAM_ADDR: usize = 0x10250000;
static NVRAM_LEN: usize = 742;

static CLM_ADDR: usize = 0x10240000;
static CLM_LEN: usize = 984;

pub struct TrngPeripherals {
    pub trng: Peri<'static, peripherals::TRNG>,
}

pub struct NetPeripherals {
    pub pio: Peri<'static, peripherals::PIO0>,
    pub dma: Peri<'static, peripherals::DMA_CH0>,
    pub pwr: Peri<'static, peripherals::PIN_23>,
    pub cs: Peri<'static, peripherals::PIN_25>,
    pub dio: Peri<'static, peripherals::PIN_24>,
    pub clk: Peri<'static, peripherals::PIN_29>,
}

pub struct Net;

impl Net {
    pub async fn init_trng(peripherals: TrngPeripherals) -> [u8; 8] {
        let trng_config = {
            let mut trng_config = trng::Config::default();

            trng_config.sample_count = 2500;

            trng_config
        };
        let mut trng = trng::Trng::new(peripherals.trng, interrupts::Irqs, trng_config);
        let mut seed = [0u8; 8];

        trng.fill_bytes(&mut seed).await;

        seed
    }

    pub async fn init_cyw43(
        peripherals: NetPeripherals,
    ) -> (
        NetDriver<'static>,
        Control<'static>,
        cyw43::Runner<'static, SpiBus<Output<'static>, PioSpi<'static, peripherals::PIO0, 0>>>,
    ) {
        let state = WIFI_STATE.init(cyw43::State::new());
        let pwr = Output::new(peripherals.pwr, gpio::Level::High);
        let cs = Output::new(peripherals.cs, gpio::Level::High);
        let mut pio = Pio::new(peripherals.pio, interrupts::Irqs);
        let dma_channel = dma::Channel::new(peripherals.dma, interrupts::Irqs);
        let spi = PioSpi::new(
            &mut pio.common,
            pio.sm0,
            cyw43_pio::DEFAULT_CLOCK_DIVIDER,
            pio.irq0,
            cs,
            peripherals.dio,
            peripherals.clk,
            dma_channel,
        );

        let fw_ptr = core::hint::black_box(FW_ADDR as *const u8);
        let firmware_slice: &[u8] = unsafe { core::slice::from_raw_parts(fw_ptr, FW_LEN) };
        let firmware_aligned: &cyw43::Aligned<cyw43::A4, [u8]> =
            unsafe { core::mem::transmute(firmware_slice) };

        let nvram_ptr = core::hint::black_box(NVRAM_ADDR as *const u8);
        let nvram_slice: &[u8] = unsafe { core::slice::from_raw_parts(nvram_ptr, NVRAM_LEN) };
        let nvram_aligned: &cyw43::Aligned<cyw43::A4, [u8]> =
            unsafe { core::mem::transmute(nvram_slice) };

        cyw43::new(state, pwr, spi, firmware_aligned, nvram_aligned).await
    }

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
        while let Err(err) = control
            .join(WIFI_SSID, JoinOptions::new(WIFI_PASSWORD.as_bytes()))
            .await
        {
            error!("join failed: {:?}", err);
            embassy_time::Timer::after_secs(1).await;
        }
    }
}

#[embassy_executor::task]
pub async fn cyw43_task(
    runner: cyw43::Runner<'static, SpiBus<Output<'static>, PioSpi<'static, peripherals::PIO0, 0>>>,
) -> ! {
    info!("net::device up");
    runner.run().await
}

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    info!("net::connection up");
    runner.run().await
}

fn apply_poland_timezone(unix_timestamp: u32) -> u32 {
    let secs_per_day = 86400;

    // 1. Rozbijamy Unix Timestamp na Rok, Miesiąc, Dzień i Godzinę UTC
    let (year, month, day, hour_utc) = unix_to_date(unix_timestamp);

    // 2. Algorytm sprawdzający, czy dla danej daty w Polsce obowiązuje DST (czas letni)
    // Zmiana czasu: ostatnia niedziela marca (UTC+2) do ostatniej niedzieli października (UTC+1)
    let is_dst = match month {
        1..=2 => false,
        3 => {
            // Marzec: zmiana w ostatnią niedzielę o 01:00 UTC (02:00 czasu lokalnego)
            // Wzór na dzień tygodnia dla 31 marca (0 = niedziela, 1 = poniedziałek... 6 = sobota)
            let w = (day_of_week(year, 3, 31) + 1) % 7;
            let last_sunday = 31 - w;
            if day > last_sunday {
                true
            } else if day == last_sunday {
                hour_utc >= 1
            } else {
                false
            }
        }
        4..=9 => true,
        10 => {
            // Październik: zmiana w ostatnią niedzielę o 01:00 UTC (03:00 czasu lokalnego)
            let w = (day_of_week(year, 10, 31) + 1) % 7;
            let last_sunday = 31 - w;
            if day < last_sunday {
                true
            } else if day == last_sunday {
                hour_utc < 1
            } else {
                false
            }
        }
        11..=12 => false,
        _ => false,
    };

    let offset_seconds = if is_dst { 2 * 3600 } else { 3600 };

    (unix_timestamp + offset_seconds) % secs_per_day
}

// Funkcja pomocnicza: Zwraca dzień tygodnia (0 = sobota, 1 = niedziela, ..., 6 = piątek) -> Algorytm Zellera
fn day_of_week(year: u32, month: u32, day: u32) -> u32 {
    let q = day;
    let mut m = month;
    let mut y = year;
    if m < 3 {
        m += 12;
        y -= 1;
    }
    let k = y % 100;
    let j = y / 100;

    (q + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7
}

// Funkcja pomocnicza: Konwersja surowych sekund Unix na podstawowe składowe daty (UTC)
fn unix_to_date(timestamp: u32) -> (u32, u32, u32, u32) {
    let secs_per_day = 86400;
    let mut days = timestamp / secs_per_day;
    let hour = (timestamp % secs_per_day) / 3600;

    // Epoka zaczyna się w czwartek, 1 stycznia 1970 roku
    let mut year = 1970;
    loop {
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if is_leap { 366 } else { 365 };
        if days >= days_in_year {
            days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0;
    while days >= month_days[month] {
        days -= month_days[month];
        month += 1;
    }

    (year, (month + 1) as u32, (days + 1), hour)
}

const NTP_PORT: u16 = 123;

#[embassy_executor::task]
pub async fn ntp_task(stack: Stack<'static>) {
    let sender = shared::CLOCK_CHANNEL.sender();
    let ntp_host = {
        let mut address = embassy_net::IpAddress::v4(127, 0, 0, 1);

        if let Ok(ips) = stack
            .dns_query("pool.ntp.org", embassy_net::dns::DnsQueryType::A)
            .await
            && let Some(ip) = ips.first()
        {
            address = *ip;
        }

        address
    };

    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];
    let rx_buffer = UDP_RX_BUFFER.init([0u8; 512]);
    let tx_buffer = UDP_TX_BUFFER.init([0u8; 512]);
    let mut socket = UdpSocket::new(stack, &mut rx_meta, rx_buffer, &mut tx_meta, tx_buffer);

    let mut local_seconds_of_day: u32 = 0;

    loop {
        let mut ntp_request = [0u8; 48];
        ntp_request[0] = 0x1B;

        let remote_endpoint = embassy_net::IpEndpoint::new(ntp_host, NTP_PORT);

        if socket.bind(0u16).is_ok() && socket.send_to(&ntp_request, remote_endpoint).await.is_ok()
        {
            // Tworzymy dedykowany, lokalny bufor o długości 48 bajtów (rozmiar pakietu NTP)
            let mut ntp_response_buffer = [0u8; 48];

            let rx_result = embassy_futures::select::select(
                socket.recv_from(&mut ntp_response_buffer), // Pożyczamy ntp_response_buffer
                embassy_time::Timer::after_secs(3),
            )
            .await;

            if let embassy_futures::select::Either::First(Ok((_, _))) = rx_result {
                let ntp_secs = u32::from_be_bytes([
                    ntp_response_buffer[40],
                    ntp_response_buffer[41],
                    ntp_response_buffer[42],
                    ntp_response_buffer[43],
                ]);

                let unix_timestamp = ntp_secs.wrapping_sub(2208988800);
                local_seconds_of_day = apply_poland_timezone(unix_timestamp);
            }
        }
        socket.close();

        // Przez najbliższą godzinę symulujemy tykanie zegara lokalnie na Core 0 i pchamy na kanał
        for _ in 0..3600 {
            let update = shared::ClockUpdate {
                hours: ((local_seconds_of_day / 3600) % 24) as u8,
                minutes: ((local_seconds_of_day / 60) % 60) as u8,
                seconds: (local_seconds_of_day % 60) as u8,
            };

            // Wysyłamy czas do Core 1. Jeśli kanał jest pełen (Core 1 akurat odświeża matrycę),
            // try_send zignoruje blokowanie i nadpisze/poczeka na następną iterację.
            let _ = sender.try_send(update);

            embassy_time::Timer::after_secs(1).await;
            local_seconds_of_day = (local_seconds_of_day + 1) % 86400;
        }
    }
}

static X_UDP_RX_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
static X_UDP_TX_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

#[embassy_executor::task]
pub async fn game_task(stack: Stack<'static>) {
    let rx_buffer = X_UDP_RX_BUFFER.init([0; 512]);
    let tx_buffer = X_UDP_TX_BUFFER.init([0; 512]);

    let mut rx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_meta = [PacketMetadata::EMPTY; 2];

    let mut socket = UdpSocket::new(stack, &mut rx_meta, rx_buffer, &mut tx_meta, tx_buffer);

    // Bindowanie lokalnego portu (0 oznacza losowy wolny port wychodzący)
    if let Err(e) = socket.bind(12346) {
        defmt::error!("Nie udało się przypisać portu UDP: {:?}", e);
        return;
    }

    defmt::info!("Zadanie sieciowe uruchomione. Klient: 192.168.0.102:12345");

    let sender = shared::GAME_CHANNEL.sender();
    // Bufor na pojedynczy przychodzący pakiet (dokładnie PACKET_SIZE = 27 bajtów)
    let mut rx_packet_buf = [0u8; game_net::PACKET_SIZE];

    loop {
        // Asynchroniczne oczekiwanie na dowolny pakiet UDP
        match socket.recv_from(&mut rx_packet_buf).await {
            Ok((size, _remote_endpoint)) => {
                // Sprawdzamy, czy pakiet ma odpowiednią długość protokołu
                if size != game_net::PACKET_SIZE {
                    defmt::warn!(
                        "Odrzucono pakiet o nieprawidłowym rozmiarze: {} bajtów",
                        size
                    );
                    continue;
                }

                // Próba deserializacji pakietu i weryfikacji sumy kontrolnej CRC
                match game_net::Packet::from_bytes(&rx_packet_buf) {
                    Ok(packet) => {
                        sender.send(packet).await;
                    }
                    Err(_) => {
                        defmt::error!("Błąd walidacji pakietu (złe MAGIC lub suma CRC)");
                    }
                }
            }
            Err(e) => {
                defmt::error!("Błąd podczas odbierania UDP: {:?}", e);
            }
        }
    }
}
