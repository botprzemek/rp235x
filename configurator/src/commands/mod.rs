mod create;
mod load;

use create::{CreateArgs, CreateCommand};
use load::{LoadArgs, LoadCommand};

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    Create(CreateArgs),
    Load(LoadArgs),
}

impl Commands {
    pub fn run(self) -> std::result::Result<(), anyhow::Error> {
        match self {
            Commands::Create(args) => CreateCommand::handle(args),
            Commands::Load(args) => LoadCommand::handle(args),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_command() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");

        let config_path = dir.path().join("config.json");
        let firmware_path = dir.path().join("firmware.bin");
        let output_path = dir.path().join("output.bin");

        std::fs::write(&firmware_path, vec![0xFF; 1000]).unwrap();

        let args = CreateArgs {
            verbose: true,
            config: config_path,
            firmware: firmware_path,
            output: output_path.clone(),
        };

        let result = CreateCommand::handle(args);
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_load_command() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");

        let config_path = dir.path().join("config.json");
        let firmware_path = dir.path().join("firmware.bin");
        let output_path = dir.path().join("output.bin");

        std::fs::write(&firmware_path, vec![0xFF; 1000]).unwrap();

        let create_args = CreateArgs {
            verbose: false,
            config: config_path,
            firmware: firmware_path,
            output: output_path.clone(),
        };
        CreateCommand::handle(create_args).expect("Failed to create config");

        let load_args = LoadArgs { path: output_path };

        let result = LoadCommand::handle(load_args);
        assert!(result.is_ok());
    }
}
