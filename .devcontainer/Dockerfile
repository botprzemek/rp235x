FROM mcr.microsoft.com/devcontainers/rust:1

RUN git config --global sequence.editor "code --wait" \
    && rustup target add thumbv8m.main-none-eabihf wasm32-unknown-unknown \
    && rustup component add llvm-tools \
    && cargo install --locked cargo-binutils cargo-watch flip-link trunk
