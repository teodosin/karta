FROM rust:1.74.1

RUN apt-get update && apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev