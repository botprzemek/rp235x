//! Linux (aarch64 / desktop) worker binary.
//!
//! Emulates the dual-core RP235x scoreboard worker on Linux:
//! - Core 0 networking (discovery, handshake, keepalive, and OTA config packets)
//! - Core 1 HUB75 ScoreboardLayout rendering and local clock interpolation
//! - Shared Game State between cores

use net::{Device, DeviceType};
use worker::sim::LinuxWorkerSimulator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==================================================");
    println!("🏀 Starting Linux / aarch64 Scoreboard Worker Simulator");
    println!("   Dual-Core Emulation: Core 0 (UDP) + Core 1 (HUB75)");
    println!("==================================================");

    let device = Device::try_new(DeviceType::Worker, "sn-linux-001", "worker-linux")?;
    let simulator = LinuxWorkerSimulator::new(device);

    simulator.run().await
}

