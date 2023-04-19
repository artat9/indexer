FROM ubuntu:20.04
RUN apt-get update && apt-get install -y \
    vim \
    curl
# Install required packages
RUN apt-get install -y build-essential
# Install Rust
ENV RUST_HOME /usr/local/lib/rust
ENV RUSTUP_HOME ${RUST_HOME}/rustup
ENV CARGO_HOME ${RUST_HOME}/cargo98
RUN mkdir /usr/local/lib/rust && \
    chmod 0755 $RUST_HOME
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > ${RUST_HOME}/rustup.sh \
    && chmod +x ${RUST_HOME}/rustup.sh \
    && ${RUST_HOME}/rustup.sh -y --default-toolchain nightly --no-modify-path
ENV PATH $PATH:$CARGO_HOME/bin
RUN rustup target add wasm32-unknown-unknown
# Install dfx
RUN sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
# Copy source
COPY ./indexer/ ./indexer/
COPY  identity.pem /root/.config/dfx/identity/default/identity.pem
COPY ./infrastructure/ ./infrastructure/
WORKDIR /infrastructure/src/deploy_canister
RUN cargo build
WORKDIR /infrastructure/src/deploy_canister/target/debug
RUN chmod +x deploy_canister
CMD ["./deploy_canister"]
