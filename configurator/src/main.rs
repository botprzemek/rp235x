use clap::builder::Str;
use clap::{Args, Parser, Subcommand};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

pub const CONFIG_OFFSET: u64 = 0x3F000;

// ---------------------------------------------------------
// Definicja CLI za pomocą Clap Derive
// ---------------------------------------------------------

#[derive(Parser)]
#[command(name = "configurator")]
#[command(about = "Narzędzie CLI do zarządzania konfiguracją w systemach wbudowanych", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Tworzy plik konfiguracyjny z JSON-a i wstrzykuje go do firmware
    Create(CreateArgs),
    /// Odczytuje i wyświetla konfigurację z gotowego pliku binarnego / firmware
    Read(ReadArgs),
}

#[derive(Args)]
struct CreateArgs {
    #[arg(default_value = str::from_utf8(CONFIG_OFFSET))]
    offset: Str,

    /// Ścieżka do pliku wejściowego JSON z konfiguracją
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    /// Ścieżka do bazowego pliku firmware.bin
    #[arg(short, long, default_value = "firmware.bin")]
    firmware: PathBuf,

    /// Ścieżka docelowa dla wygenerowanego firmware z wstrzykniętym configiem
    #[arg(default_value = "firmware_configured.bin")]
    output: PathBuf,
}

#[derive(Args)]
struct ReadArgs {
    /// Ścieżka do pliku binarnego/firmware zawierającego wstrzykniętą konfigurację
    #[arg(default_value = "firmware_configured.bin")]
    path: PathBuf,
}

// ---------------------------------------------------------
// Logika wykonawcza
// ---------------------------------------------------------

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create(args) => {
            let json_path = args.config;
            let input_firmware = args.firmware;
            let output_firmware = args.output;

            let config = if json_path.exists() {
                println!("> Odczytuję konfigurację z: {}", json_path.display());
                let content = fs::read_to_string(&json_path).expect("Błąd odczytu pliku JSON");
                let input: ConfigInput =
                    serde_json::from_str(&content).expect("Błąd parsowania JSON");

                DeviceConfig::new(
                    &input.serial_number,
                    &input.device_id,
                    &input.wifi_ssid,
                    &input.wifi_pass,
                    &input.firmware_ver,
                )
                .unwrap()
            } else {
                println!(
                    "> Brak pliku JSON ({}), generuję domyślny config testowy.",
                    json_path.display()
                );
                DeviceConfig::new(
                    "SN-DEFAULT-00",
                    "scoreboard-main",
                    "MojaSiecWiFi",
                    "TajneHaslo123",
                    "1.0.0",
                )
                .unwrap()
            };

            if input_firmware.exists() {
                println!(
                    "> Kopiuję bazowy firmware: {} -> {}",
                    input_firmware.display(),
                    output_firmware.display()
                );
                fs::copy(&input_firmware, &output_firmware)
                    .expect("Nie udało się skopiować pliku firmware");

                let mut file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&output_firmware)
                    .expect("Nie udało się otworzyć wyjściowego firmware");

                file.seek(SeekFrom::Start(CONFIG_OFFSET))
                    .expect("Offset wykracza poza rozmiar pliku firmware");

                file.write_all(config.as_bytes())
                    .expect("Nie udało się zapisać konfiguracji do firmware");

                println!(
                    "> Sukces! Wstrzyknięto konfigurację na offset: 0x{:X}",
                    CONFIG_OFFSET
                );
            } else {
                let standalone_path = "firmware_configured.bin";
                let mut out_file =
                    File::create(standalone_path).expect("Nie udało się utworzyć pliku");

                // Przesuwamy się na wymagany offset i dopiero tam zapisujemy konfigurację,
                // dzięki czemu plik ma właściwy rozmiar w pamięci masowej.
                out_file
                    .seek(SeekFrom::Start(CONFIG_OFFSET))
                    .expect("Nie udało się ustawić offsetu w nowym pliku");
                out_file
                    .write_all(config.as_bytes())
                    .expect("Nie udało się zapisać konfiguracji");

                println!(
                    "> Brak pliku {} – wygenerowano plik z paddingiem: {}",
                    input_firmware.display(),
                    standalone_path
                );
            }

            println!(
                "> Rozmiar struktury configu: {} bajtów",
                std::mem::size_of::<DeviceConfig>()
            );
        }

        Commands::Read(args) => {
            let target_path = args.path;

            if !target_path.exists() {
                eprintln!("> Błąd: Plik {} nie istnieje!", target_path.display());
                std::process::exit(1);
            }

            println!(
                "> Odczytuję konfigurację z pliku: {}",
                target_path.display()
            );
            let mut file =
                File::open(&target_path).expect("Nie udało się otworzyć pliku do odczytu");

            // Przechodzimy do miejsca, gdzie zapisujemy konfigurację
            file.seek(SeekFrom::Start(CONFIG_OFFSET))
                .expect("Nie udało się odnaleźć offsetu w pliku");

            let struct_size = std::mem::size_of::<DeviceConfig>();
            let mut buffer = vec![0u8; struct_size];
            file.read_exact(&mut buffer)
                .expect("Nie udało się odczytać struktury konfiguracji z podanego offsetu");

            let config =
                DeviceConfig::from_bytes(&buffer).expect("Błąd parsowania surowych bajtów");

            println!("\n=== ZNALEZIONA KONFIGURACJA URZĄDZENIA ===");
            println!("- Serial Number: {}", config.get_serial_string());
            println!("- Device ID:     {}", config.get_device_id_string());
            println!("- WiFi SSID:     {}", config.get_wifi_ssid_string());
            println!("- Firmware Ver:  {}", config.get_firmware_ver_string());
            println!("- Offset w pliku: 0x{:X}", CONFIG_OFFSET);
            println!("==========================================\n");
        }
    }
}
