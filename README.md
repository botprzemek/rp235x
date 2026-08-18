# rp235x

```sh
cargo install probe-rs

# firmware
probe-rs download bin/cyw43/43439A0.bin --binary-format bin --chip RP235x --base-address 0x10200000

# clm
probe-rs download bin/cyw43/43439A0_clm.bin --binary-format bin --chip RP235x --base-address 0x10240000

# nvram
probe-rs download bin/cyw43/nvram_rp2040.bin --binary-format bin --chip RP235x --base-address 0x10250000
```

build

```sh
cargo build --release
```

dev

```sh
cargo install cargo-watch
cargo watch --watch "src" --exec "run"
```