# rp235x

cargo install probe-rs

probe-rs download bin/cyw43/43439A0.bin --binary-format bin --chip RP235x --base-address 0x10200000
probe-rs download bin/cyw43/43439A0_clm.bin --binary-format bin --chip RP235x --base-address 0x10240000
probe-rs download bin/cyw43/nvram_rp2040.bin --binary-format bin --chip RP235x --base-address 0x10250000

build

cargo build
cargo run

dev

cargo install cargo-watch
cargo watch --watch "src" --exec "run"