Bevy wrapper for [File System Graph](https://github.com/teodosin/fs_graph).

Possibly could evolve into the whole backend layer of Karta. 

#### Brainstorming a plugin structure

The job of this crate is to use fs_graph to load and deload nodes and edges as bevy entities. 

Startup systems
* Init Graph
    * load vault from config
    * Graph as a Resource