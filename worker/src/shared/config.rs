use config::Config;

pub fn read() -> &'static Config {
    Config::read().unwrap()
}
