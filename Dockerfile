FROM rust:1.74.1

RUN apt-get update && apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
RUN apt-get install -y libxcursor-dev libxrandr-dev libxi-dev libx11-xcb-dev libvulkan-dev

RUN adduser developer --disabled-password
ENV LD_LIBRARY_PATH=/usr/lib/wsl/lib

