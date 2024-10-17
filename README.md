# Karta
![Demo gif](/docs/karta.gif)

> [!IMPORTANT] 
> This project is in its very early stages and is therefore highly experimental, barely usable and not really useful yet. Most of the mentioned features are planned but not yet implemented. The version shown in the gif above is in the v.0.0.1_legacy branch. 

## Introduction

**Karta** is a node-based file explorer and compositor. It creates a network out of a selected section of the file system and allows for files and folders to be arranged spatially and for arbitrary connections to be made between them. Attributes may be added to any node or connection. This network could then be queried for various purposes, though originally intended for structuring creative projects and making media art. 

The project is free and open-sourced under a GPL license. 

For a more detailed explanation of the project's purpose and goals, refer to docs/vision.md. For technical details refer to docs/architecture.md. 

Key features:
* Local first - the files and network database exist locally on your machine. No lock-in. The goal is to keep the storage format well documented and allow for syncing and exporting to plain text files. 
* Contextual - nodes don't have absolute spatial positions, but rather contextual ones. Since the network is always viewed from the point of view or "context" of some individual node, the positions of its connections are always relative to it. Two nodes can be positioned differently relative to each other depending on which context you look from.
* Virtual nodes - not all nodes have to physically exist in the file system. You can create "virtual" nodes of different types that get stored directly in the database. 

## Project Structure

The core of the project is karta_server. It's a local HTTP server responsible for managing the underlying graph structure. Multiple clients can read from and write to it. 

Bevy_karta_client is a Karta client for the Bevy game engine. Bevy_karta_ui is the corresponding ui layer. Karta_bevy is the standalone application that combines these. These are all incomplete, but functional, and may be expanded upon. 

## Contributing

It's much too early for me to ask or hope for contributions. The most valuable thing you might contribute at this stage is sharing your thoughts about the project and discussing it with me, to help clarify the path forward. I am active in the Bevy discord, so you may find me there under the same username. 

Use the develop branch for the most up-to-date version. Main is for stable-ish releases. 

## Development

### Development within docker container

For those who don't want to install the development environment directly to their computers, 
it is possible to develop Karta within an isolated [Docker](https://docs.docker.com/) container. 
You can build the image and run the container from the project root directory by typing: 

    docker compose up

This command will build the `karta-rust-devenv` docker image, if it does not already exist, and 
start the container. After that it is possible to run e.g. VSCode and connect to this running container by 
executing the "Dev Containers: Attach to running container" command, and chose the correct container. 
This will open a new VSCode instance which is now running within our rust development container. 
If you install rust- or any other plugin to VSCode, it will be valid only when running this container. 

The project directory is to be found in `/project` directory. Use the terminal from VSCode to run 
any command within the context of docker container. 

Resources:
- https://github.com/bevyengine/bevy/issues/11768 
- https://github.com/bevyengine/bevy/discussions/4953#discussioncomment-8571666

#### Running from docker container

First you have to [install NVIDIA container toolkit on host](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html), 
The script is for example: 

    curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | sudo gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg \
    && curl -s -L https://nvidia.github.io/libnvidia-container/stable/deb/nvidia-container-toolkit.list | \
    sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | \
    sudo tee /etc/apt/sources.list.d/nvidia-container-toolkit.list
    sudo apt-get install -y nvidia-container-toolkit
    sudo nvidia-ctk runtime configure --runtime=docker


