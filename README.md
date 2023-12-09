# Karta

Karta is an experimental creative application built on top of a node-based file browser. It is very bare-bones and not ready for use quite yet. Root paths are hardcoded so the app will not work on your machine without modification. 

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


