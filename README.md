# Karta

#### Disclaimer

Karta is an experimental creative application built on top of a node-based file browser. It is very bare and unstable, with only some of the foundations set in place.  It is a work in progress and not ready for even remotely serious use, but feel free to look around and try it. Root paths are hardcoded so the app will not work on your machine without modification.

Early development has been quite chaotic and flow state driven, and this is reflected in the code and commits. Steps are being taken to steer the development process to be more systematic and understandable, but that will take time and some learning on my part. 

## Introduction

Welcome to **Karta**, a budding project currently in development, aimed at rethinking file management and visualization for creative workflows. This early-stage application introduces a novel concept: a node-based file browser, designed to visually represent the intricate network of files and folders within digital projects.

At this stage, Karta is a prototype, exploring the potential to view and manage files as part of an interconnected graph. Each file and folder is represented as a node, and the relationships between them are visualized as links. This approach is not just about organizing files; it's about understanding the structure and interdependencies of your project at a glance.

For a more detailed explanation of the project's purpose and goals, refer to VISION.md. [[VISION.md]]

## Getting Started

* Make sure you have Rust installed. Karta uses the Bevy game engine so familiarity with it is recommended. 
* Clone the repo
* Build and run 

## Usage

Once your root folder is setup, running the app will display the folder and its contents as nodes in a force-directed graph. Middle-mouse click to pan the view and scroll to zoom. Dragging from the edge of a node to another will create a new connection between those nodes. Right-clicking on a node will bring up a menu where you can pin and unpin nodes (to be ignored by the force simulation) and move to another nodes' context. 

## Contributing

It's much too early for me to ask or hope for contributions. The most valuable thing you might contribute at this stage is sharing your thoughts about the project and discussing it with me, to help clarify the path forward. I am active in the Bevy discord, so you may find me there under the same username. 
Karta is an experimental creative application built on top of a node-based file browser. It is very bare-bones and not ready for use quite yet. 

Use the develop branch for the most up-to-date version. 

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


