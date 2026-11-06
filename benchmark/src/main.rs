//! Master benchmark and load simulator CLI.

use std::fs::{self, File};
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

use clap::Parser;
use futures::{SinkExt, StreamExt};
use net::data::device::model::{Device, DeviceType};
use net::layout::PACKET_SIZE;
use net::{ClientEvent, ClientPacket};
use tokio::net::UdpSocket;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

/// Benchmark and load simulator for the master server with exponential scaling.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "High-performance exponential UDP & WS stress test for master", long_about = None)]
pub struct Cli {
    /// Target UDP address (host:port)
    #[arg(long, default_value = "127.0.0.1:9000")]
    pub udp_host: String,

    /// Target WebSocket URL
    #[arg(long, default_value = "ws://127.0.0.1:3000/ws")]
    pub ws_url: String,

    /// Initial number of workers (step 0: N = start_workers)
    #[arg(short = 'w', long, default_value_t = 2)]
    pub start_workers: usize,

    /// Maximum workers limit to avoid unbound tests
    #[arg(long, default_value_t = 65536)]
    pub max_workers: usize,

    /// Scaling multiplier for each iteration (e.g. 2 for 2^k or 4 for (2^k)^2)
    #[arg(short = 'm', long, default_value_t = 2)]
    pub scale_multiplier: usize,

    /// Sending rate per UDP worker in Hz
    #[arg(short = 'r', long, default_value_t = 50)]
    pub udp_rate: u64,

    /// Number of simulated HMI / browser WebSocket clients per stage
    #[arg(short = 'c', long, default_value_t = 5)]
    pub ws_clients: usize,

    /// Duration of each benchmark stage in seconds
    #[arg(short = 'd', long, default_value_t = 10)]
    pub duration: u64,

    /// Timeout threshold in seconds (no reply from master for this duration considered timeout)
    #[arg(short = 't', long, default_value_t = 3)]
    pub timeout: u64,

    /// Directory where stage logs and summary reports will be written
    #[arg(long, default_value = "benchmark_logs")]
    pub log_dir: String,
}

#[derive(Debug, Clone)]
pub struct StageResult {
    pub workers: usize,
    pub elapsed_secs: f64,
    pub udp_sent: u64,
    pub udp_recv: u64,
    pub ws_sent: u64,
    pub ws_recv: u64,
    pub ws_errors: u64,
    pub timed_out: bool,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let udp_target: SocketAddr = args
        .udp_host
        .parse()
        .expect("Invalid UDP host address, expected format ip:port (e.g. 127.0.0.1:9000)");

    let log_dir_path = PathBuf::from(&args.log_dir);
    if let Err(e) = fs::create_dir_all(&log_dir_path) {
        eprintln!("Failed to create log directory {}: {e}", log_dir_path.display());
    }

    println!("==================================================");
    println!("🚀 Starting Master Exponential Load Benchmark");
    println!("   UDP Target:         {}", udp_target);
    println!("   WS Target:          {}", args.ws_url);
    println!("   Initial Workers:    {}", args.start_workers);
    println!("   Multiplier:         x{}", args.scale_multiplier);
    println!("   Max Workers:        {}", args.max_workers);
    println!("   UDP Send Rate:      {} Hz/worker", args.udp_rate);
    println!("   WS Clients:         {}", args.ws_clients);
    println!("   Stage Duration:     {} s", args.duration);
    println!("   Timeout Limit:      {} s without response", args.timeout);
    println!("   Logs Directory:     {}", log_dir_path.display());
    println!("==================================================\n");

    let mut current_workers = args.start_workers.max(1);
    let mut stage = 1;
    let mut results: Vec<StageResult> = Vec::new();

    loop {
        println!("--------------------------------------------------");
        println!("▶️  STAGE #{stage}: Testing with {current_workers} workers...");
        println!("--------------------------------------------------");

        let result = run_stage(
            stage,
            current_workers,
            udp_target,
            &args.ws_url,
            args.udp_rate,
            args.ws_clients,
            Duration::from_secs(args.duration),
            Duration::from_secs(args.timeout),
            &log_dir_path,
        )
        .await;

        let is_timeout = result.timed_out;
        results.push(result);

        if is_timeout {
            println!("\n🚨 TIMEOUT DETECTED! Master stopped responding in stage #{stage} ({current_workers} workers).");
            break;
        }

        if current_workers >= args.max_workers {
            let max = args.max_workers;
            println!("\n🏁 Reached maximum configured worker count ({max}). Ending benchmark.");
            break;
        }

        let next_workers = current_workers.saturating_mul(args.scale_multiplier.max(2));
        if next_workers == current_workers {
            break;
        }
        current_workers = next_workers.min(args.max_workers);
        stage += 1;

        // Cool-down period between stages
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    save_summary_report(&results, &log_dir_path);
}

async fn run_stage(
    stage: usize,
    workers: usize,
    udp_target: SocketAddr,
    ws_url: &str,
    udp_rate: u64,
    ws_clients: usize,
    duration: Duration,
    timeout_duration: Duration,
    log_dir: &PathBuf,
) -> StageResult {
    let running = Arc::new(AtomicBool::new(true));
    let udp_sent = Arc::new(AtomicU64::new(0));
    let udp_recv = Arc::new(AtomicU64::new(0));
    let ws_sent = Arc::new(AtomicU64::new(0));
    let ws_recv = Arc::new(AtomicU64::new(0));
    let ws_errors = Arc::new(AtomicU64::new(0));

    // Socket pool auto-scaled (max 64 FDs to stay well below system limits)
    let socket_pool_size = (workers / 64 + 1).clamp(1, 64);
    let mut socket_pool = Vec::with_capacity(socket_pool_size);

    for idx in 0..socket_pool_size {
        match UdpSocket::bind("0.0.0.0:0").await {
            Ok(socket) => {
                let shared_socket = Arc::new(socket);
                let rx_socket = Arc::clone(&shared_socket);
                let rx_running = Arc::clone(&running);
                let rx_recv = Arc::clone(&udp_recv);

                tokio::spawn(async move {
                    let mut buf = [0u8; PACKET_SIZE];
                    while rx_running.load(Ordering::Relaxed) {
                        match tokio::time::timeout(Duration::from_millis(200), rx_socket.recv_from(&mut buf)).await {
                            Ok(Ok((size, _))) if size == PACKET_SIZE => {
                                rx_recv.fetch_add(1, Ordering::Relaxed);
                            }
                            _ => {}
                        }
                    }
                });

                socket_pool.push(shared_socket);
            }
            Err(e) => {
                eprintln!("Failed to bind shared UDP socket {idx}: {e}");
            }
        }
    }

    let mut handles = Vec::new();

    // Spawn UDP workers
    for worker_idx in 0..workers {
        let running_clone = Arc::clone(&running);
        let sent_clone = Arc::clone(&udp_sent);
        let socket = Arc::clone(&socket_pool[worker_idx % socket_pool.len()]);

        let handle = tokio::spawn(async move {
            run_udp_worker(worker_idx, socket, udp_target, udp_rate, running_clone, sent_clone).await;
        });
        handles.push(handle);
    }

    // Spawn WebSocket clients
    for client_idx in 0..ws_clients {
        let running_clone = Arc::clone(&running);
        let sent_clone = Arc::clone(&ws_sent);
        let recv_clone = Arc::clone(&ws_recv);
        let errors_clone = Arc::clone(&ws_errors);
        let url = ws_url.to_string();

        let handle = tokio::spawn(async move {
            run_ws_client(client_idx, url, running_clone, sent_clone, recv_clone, errors_clone).await;
        });
        handles.push(handle);
    }

    let start_time = Instant::now();
    let mut last_recv_udp = 0u64;
    let mut last_activity = Instant::now();
    let mut timed_out = false;

    // Monitor for timeout or stage end
    while start_time.elapsed() < duration {
        tokio::time::sleep(Duration::from_millis(250)).await;

        let current_recv_udp = udp_recv.load(Ordering::Relaxed);
        if current_recv_udp > last_recv_udp {
            last_recv_udp = current_recv_udp;
            last_activity = Instant::now();
        } else if last_activity.elapsed() >= timeout_duration && start_time.elapsed() >= timeout_duration {
            timed_out = true;
            break;
        }
    }

    running.store(false, Ordering::Relaxed);

    for handle in handles {
        let _ = handle.await;
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let total_udp_sent = udp_sent.load(Ordering::Relaxed);
    let total_udp_recv = udp_recv.load(Ordering::Relaxed);
    let total_ws_sent = ws_sent.load(Ordering::Relaxed);
    let total_ws_recv = ws_recv.load(Ordering::Relaxed);
    let total_ws_err = ws_errors.load(Ordering::Relaxed);

    let stage_result = StageResult {
        workers,
        elapsed_secs: elapsed,
        udp_sent: total_udp_sent,
        udp_recv: total_udp_recv,
        ws_sent: total_ws_sent,
        ws_recv: total_ws_recv,
        ws_errors: total_ws_err,
        timed_out,
    };

    println!("   ⏱  Elapsed:        {:.2} s", elapsed);
    println!("   📦 UDP Sent:       {} ({:.1} pps)", total_udp_sent, total_udp_sent as f64 / elapsed);
    println!("   📥 UDP Recv:       {} ({:.1} pps)", total_udp_recv, total_udp_recv as f64 / elapsed);
    println!("   📡 WS Recv:        {} ({:.1} msg/s)", total_ws_recv, total_ws_recv as f64 / elapsed);
    if total_ws_err > 0 {
        println!("   ⚠️  WS Errors:      {}", total_ws_err);
    }
    if timed_out {
        println!("   ❌ Stage Status:    TIMEOUT (No packets received for >{:?})", timeout_duration);
    } else {
        println!("   ✅ Stage Status:    PASSED");
    }

    // Save detailed stage log file
    save_stage_log(stage, &stage_result, log_dir);

    stage_result
}

fn save_stage_log(stage: usize, result: &StageResult, log_dir: &PathBuf) {
    let filename = format!("stage_{:02}_{}_workers.log", stage, result.workers);
    let file_path = log_dir.join(filename);

    if let Ok(mut f) = File::create(&file_path) {
        let _ = writeln!(f, "=== Benchmark Stage #{stage} Log ===");
        let _ = writeln!(f, "Date:           {:?}", SystemTime::now());
        let _ = writeln!(f, "Workers:        {}", result.workers);
        let _ = writeln!(f, "Elapsed:        {:.2} s", result.elapsed_secs);
        let _ = writeln!(f, "UDP Sent:       {} ({:.1} pps)", result.udp_sent, result.udp_sent as f64 / result.elapsed_secs);
        let _ = writeln!(f, "UDP Recv:       {} ({:.1} pps)", result.udp_recv, result.udp_recv as f64 / result.elapsed_secs);
        let _ = writeln!(f, "UDP Delivery:   {:.2}%", if result.udp_sent > 0 { (result.udp_recv as f64 / result.udp_sent as f64) * 100.0 } else { 0.0 });
        let _ = writeln!(f, "WS Sent:        {} ({:.1} msg/s)", result.ws_sent, result.ws_sent as f64 / result.elapsed_secs);
        let _ = writeln!(f, "WS Recv:        {} ({:.1} msg/s)", result.ws_recv, result.ws_recv as f64 / result.elapsed_secs);
        let _ = writeln!(f, "WS Errors:      {}", result.ws_errors);
        let _ = writeln!(f, "Status:         {}", if result.timed_out { "TIMEOUT" } else { "SUCCESS" });
    }
}

fn save_summary_report(results: &[StageResult], log_dir: &PathBuf) {
    let file_path = log_dir.join("summary_report.md");

    if let Ok(mut f) = File::create(&file_path) {
        let _ = writeln!(f, "# Master Load Benchmark Summary Report\n");
        let _ = writeln!(f, "| Stage | Workers | UDP Sent (pps) | UDP Recv (pps) | Loss % | WS Recv (msg/s) | Status |");
        let _ = writeln!(f, "|-------|---------|----------------|----------------|--------|-----------------|--------|");

        for (idx, r) in results.iter().enumerate() {
            let pps_sent = r.udp_sent as f64 / r.elapsed_secs;
            let pps_recv = r.udp_recv as f64 / r.elapsed_secs;
            let loss = if r.udp_sent > 0 {
                ((r.udp_sent.saturating_sub(r.udp_recv)) as f64 / r.udp_sent as f64) * 100.0
            } else {
                0.0
            };
            let ws_msg = r.ws_recv as f64 / r.elapsed_secs;
            let status = if r.timed_out { "❌ TIMEOUT" } else { "✅ PASSED" };

            let _ = writeln!(
                f,
                "| #{:02} | {} | {:.1} | {:.1} | {:.1}% | {:.1} | {} |",
                idx + 1,
                r.workers,
                pps_sent,
                pps_recv,
                loss,
                ws_msg,
                status
            );
        }

        println!("\n📄 Saved detailed stage logs and summary report to:");
        println!("   {}", file_path.display());
    }
}

async fn run_udp_worker(
    id: usize,
    socket: Arc<UdpSocket>,
    target: SocketAddr,
    rate_hz: u64,
    running: Arc<AtomicBool>,
    sent: Arc<AtomicU64>,
) {
    let serial = format!("RP235X_{:04}", id);
    let device_id = format!("DEV_{:04}", id);
    let device = Device::try_new(DeviceType::Worker, &serial, &device_id).expect("Valid device config");

    let interval = Duration::from_micros(1_000_000 / rate_hz.max(1));
    let mut seq = 0u8;

    while running.load(Ordering::Relaxed) {
        let packet = ClientPacket::new(ClientEvent::HandshakeHeartbeat, seq, device.to_bytes());
        seq = seq.wrapping_add(1);

        if socket.send_to(&packet.to_bytes(), target).await.is_ok() {
            sent.fetch_add(1, Ordering::Relaxed);
        }

        tokio::time::sleep(interval).await;
    }
}

async fn run_ws_client(
    id: usize,
    url: String,
    running: Arc<AtomicBool>,
    sent: Arc<AtomicU64>,
    recv: Arc<AtomicU64>,
    errors: Arc<AtomicU64>,
) {
    let ws_stream = match connect_async(&url).await {
        Ok((stream, _)) => stream,
        Err(e) => {
            eprintln!("Failed to connect WS client {id} to {url}: {e}");
            errors.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Perform browser handshake
    let serial = format!("BROWSER_{:04}", id);
    let device_id = format!("HMI_{:04}", id);
    let device = match Device::try_new(DeviceType::WebBrowser, &serial, &device_id) {
        Ok(d) => d,
        Err(_) => return,
    };

    let handshake = ClientPacket::new(ClientEvent::HandshakeRequest, 0, device.to_bytes());
    if write.send(Message::Binary(bytes::Bytes::copy_from_slice(&handshake.to_bytes()))).await.is_err() {
        errors.fetch_add(1, Ordering::Relaxed);
        return;
    }
    sent.fetch_add(1, Ordering::Relaxed);

    // Read loop
    let read_running = Arc::clone(&running);
    let read_recv = Arc::clone(&recv);
    let read_errors = Arc::clone(&errors);

    tokio::spawn(async move {
        while read_running.load(Ordering::Relaxed) {
            match tokio::time::timeout(Duration::from_millis(200), read.next()).await {
                Ok(Some(Ok(Message::Binary(data)))) if data.len() == PACKET_SIZE => {
                    read_recv.fetch_add(1, Ordering::Relaxed);
                }
                Ok(Some(Err(_))) => {
                    read_errors.fetch_add(1, Ordering::Relaxed);
                    break;
                }
                _ => {}
            }
        }
    });

    // Keepalive / Heartbeat
    let mut seq = 1u8;
    while running.load(Ordering::Relaxed) {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let ping = ClientPacket::new(ClientEvent::HandshakeHeartbeat, seq, device.to_bytes());
        seq = seq.wrapping_add(1);
        if write.send(Message::Binary(bytes::Bytes::copy_from_slice(&ping.to_bytes()))).await.is_ok() {
            sent.fetch_add(1, Ordering::Relaxed);
        } else {
            errors.fetch_add(1, Ordering::Relaxed);
            break;
        }
    }
}
