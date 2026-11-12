use std::io::{self, Write};
use std::time::Duration;

use clap::Args;
use config::ConfigError;
use config::probe_rs::Permissions;
use config::probe_rs::probe::list::Lister;
use config::probe_rs::rtt::Rtt;

#[derive(Args)]
#[command(about = "Attach to the running RP2350 target and stream RTT logs", long_about = None)]
pub struct AttachArgs {

    #[arg(short, long, default_value_t = 5000)]
    pub speed: u32,
}

pub struct AttachCommand;

impl AttachCommand {

    pub fn handle(args: &AttachArgs) -> Result<(), ConfigError> {
        let lister = Lister::new();
        let probes = lister.list_all();

        if probes.is_empty() {
            eprintln!("Error: no debug probes found");
            return Err(ConfigError::FileNotFound);
        }

        println!("Opening probe: {}", probes[0].identifier);
        let mut probe = lister.open(&probes[0]).map_err(|error| {
            eprintln!("Failed to open debug probe: {error:?}");
            ConfigError::FileNotFound
        })?;

        let _ = probe.set_speed(args.speed);

        println!("Attaching to RP2350 target (under-reset = false)...");
        let mut session = probe
            .attach("RP2350", Permissions::default())
            .map_err(|error| {
                eprintln!("Failed to attach to RP2350 target: {error:?}");
                ConfigError::FileNotFound
            })?;

        println!("Attached to RP2350. Looking for RTT control block...");

        let mut core = session.core(0).map_err(|error| {
            eprintln!("Failed to access core 0: {error:?}");
            ConfigError::FileNotFound
        })?;

        let mut rtt = loop {
            match Rtt::attach(&mut core) {
                Ok(rtt) => {
                    println!("RTT control block found! Streaming logs (Ctrl+C to stop)...");
                    break Some(rtt);
                }
                Err(_error) => {

                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }
        };

        let mut stdout = io::stdout();
        let mut buffer = [0u8; 1024];

        if let Some(ref mut rtt_handle) = rtt {
            let channels = rtt_handle.up_channels();
            if let Some(up_channel) = channels.get_mut(0) {
                loop {
                    match up_channel.read(&mut core, &mut buffer) {
                        Ok(count) if count > 0 => {
                            let _ = stdout.write_all(&buffer[..count]);
                            let _ = stdout.flush();
                        }
                        Ok(_) => {
                            std::thread::sleep(Duration::from_millis(10));
                        }
                        Err(e) => {
                            eprintln!("RTT read error: {e:?}");
                            break;
                        }
                    }
                }
            } else {
                println!("No RTT up channel 0 found. Target is running.");
            }
        }

        Ok(())
    }
}
