FROM rust:latest AS builder

# Copy local code to the container image.
WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY pdsmigration-common pdsmigration-common
COPY pdsmigration-gui pdsmigration-gui
COPY pdsmigration-web pdsmigration-web

RUN cargo build --release --package pdsmigration-web

FROM rust:slim

COPY --from=builder /app/target/release/pdsmigration-web/ .

ENTRYPOINT ["./pdsmigration-web"]