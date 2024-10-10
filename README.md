# Karta
![Demo gif](/docs/karta.gif)

#### Disclaimer

This readme hasn't been updated since the monorepo was established. 

This project is in its very early stages and is therefore highly experimental, barely usable and not really useful yet. Most of the mentioned features are planned but not yet implemented. The version shown in the gif above is in the v.0.0.1_legacy branch. 

## Introduction

**Karta** is a node-based file explorer and compositor. It creates a network out of a selected section of the file system and allows for files and folders to be arranged spatially and for arbitrary connections to be made between them. Attributes may be added to any node or connection. This network could then be queried for various purposes, though chiefly intended for structuring creative projects and making media art. 

The project is free and open-sourced under a GPL license. 

For a more detailed explanation of the project's purpose and goals, refer to docs/vision.md. For technical details refer to docs/architecture.md. 

Key features:
* Local first - the files and network database exist locally on your machine. No lock-in. The goal is to keep the storage format well documented and allow for syncing and exporting to plain text files.
* Contextual - nodes don't have absolute spatial positions, but rather contextual ones. Since the network is always viewed from the point of view or "context" of some individual node, the positions of its connections are always relative to it. Two nodes can be positioned differently relative to each other depending on which context you look from.
* Simulated - node arrangement may be force simulated. Currently not much more than a novelty, but the idea is to provide a varied selection of arrangement tools to free the user up to focus more on the content, especially when creating many nodes at once. 
* Virtual nodes - not all nodes have to physically exist in the file system. You can create "virtual" nodes of different types that get stored directly in the database. 

Plans / wishlist
* Operator nodes - a system for creating functional nodes with inputs and outputs that can manipulate data, like in other node-based programming tools. Useful for quick edits and  experimentation right in the file explorer. 
* Rich previews - a system for adding custom preview generation support for any file type. 
* Composition - the ability to package a selection or sequence of nodes into a file, such as a series of images into a pdf or gif. 
* Presentation - support for displaying the previews of active nodes in the graph background or a separate window. 
* More dreamy ideas over in docs/vision.md

## Getting Started

* Make sure you have Rust installed. Karta uses the Bevy game engine so familiarity with it is recommended. 
* Clone the repo
* Build and run 

## Usage

At first startup, you will be asked to choose a folder to create your vault in. Once set up, the contents of that folder will be spawned in as a force-directed graph. Middle-mouse click to pan the view and scroll to zoom. Dragging from the edge of a node to another will create a new connection between those nodes. Right-clicking on a node will bring up a menu where you can pin and unpin nodes (to be ignored by the force simulation) and move to another nodes' context. 

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


