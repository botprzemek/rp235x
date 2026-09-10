use std::path::PathBuf;

use config::{
    Config,
    file::{FileReader, FileWriter},
    print::Print,
};

use anyhow::{Error, anyhow};
use clap::Args;

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct CreateArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long, default_value = "config.json")]
    pub input: PathBuf,

    #[arg(default_value = "config.bin")]
    pub output: PathBuf,
}

pub struct CreateCommand;

impl CreateCommand {
    pub fn handle(args: CreateArgs) -> Result<(), Error> {
        let config = Config::read_json(&args.input)?;

        if args.verbose {
            config.print()?;
        }

        let aes_hex = std::env::var("AES_SECRET_KEY")
            .map_err(|_| anyhow!("Zmienna środowiskowa AES_SECRET_KEY nie jest ustawiona!"))?;
        let mut encryption_key = [0u8; 32];
        hex::decode_to_slice(aes_hex.trim(), &mut encryption_key)
            .map_err(|e| anyhow!("Nieprawidłowy format hex klucza AES: {}", e))?;

        let ed_hex = std::env::var("ED25519_SECRET_KEY")
            .map_err(|_| anyhow!("Zmienna środowiskowa ED25519_SECRET_KEY nie jest ustawiona!"))?;
        let mut signing_key = [0u8; 32];
        hex::decode_to_slice(ed_hex.trim(), &mut signing_key)
            .map_err(|e| anyhow!("Nieprawidłowy format hex klucza Ed25519: {}", e))?;

        config.write_bin(&args.output, &signing_key, &encryption_key)?;

        Ok(())
    }
}
