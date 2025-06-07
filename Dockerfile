# syntax=docker/dockerfile:1
FROM rust:slim as builder
WORKDIR /src
COPY . .
RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    libdbus-1-dev \
    libpipewire-0.3-dev \
    libasound2-dev \
    libgtk-3-dev \
    libx11-dev \
    pkg-config \
    build-essential \
    cmake \
    libssl-dev \
    libzstd-dev \
    libudev-dev
RUN cargo build --release

FROM scratch
COPY --from=builder /src/target/release/hidemyweeb /
ENTRYPOINT ["/hidemyweeb"]
