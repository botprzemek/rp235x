FROM mcr.microsoft.com/devcontainers/rust:1

RUN rustup target add thumbv8m.main-none-eabihf wasm32-unknown-unknown \
    && rustup component add llvm-tools \
    && cargo install --locked cargo-binutils cargo-watch flip-link trunk \
    && chmod -R a+w "$CARGO_HOME"

USER vscode

RUN git config --global sequence.editor "code --wait"
