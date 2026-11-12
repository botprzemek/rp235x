use worker::core0::Core0;
use worker::core1::Core1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==================================================");
    println!("🏀 Starting Linux Scoreboard Worker");
    println!("   Dual-Core Architecture: Core 0 (UDP) + Core 1 (HUB75)");
    println!("==================================================");

    let core1_thread = std::thread::Builder::new()
        .name("core1".into())
        .spawn(Core1::entry)?;

    let core0_thread = std::thread::Builder::new()
        .name("core0".into())
        .spawn(Core0::entry)?;

    core1_thread.join().expect("Core 1 thread joined");
    core0_thread.join().expect("Core 0 thread joined");

    Ok(())
}
