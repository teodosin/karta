FROM rust:1.76.0

# Install cuda-toolkit
RUN distribution=$(. /etc/os-release;echo $ID$VERSION_ID | sed -e 's/\.//g') && \
    wget https://developer.download.nvidia.com/compute/cuda/repos/$distribution/x86_64/cuda-keyring_1.1-1_all.deb && \
    dpkg -i cuda-keyring_1.1-1_all.deb && \
    export DEBIAN_FRONTEND=noninteractive && \
    apt-get -y update && \
    apt-get -y install cuda-toolkit && \
    rm cuda*

RUN export DEBIAN_FRONTEND=noninteractive && apt-get -y upgrade \
    && apt-get -y install --no-install-recommends \
    libasound2-dev \
    libasound2-plugins \
    libudev-dev


WORKDIR /project

CMD ["bash"]
