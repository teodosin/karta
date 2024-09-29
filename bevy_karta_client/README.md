Bevy wrapper for [File System Graph](https://github.com/teodosin/karta_server).

Possibly could evolve into the whole backend layer of Karta. 

#### Brainstorming a plugin structure

The job of this crate is to use karta_server to load and deload nodes and edges as bevy entities. 

Startup systems
* Init Graph
    * load vault from config
    * Graph as a Resource