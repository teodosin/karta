This document outlines some of the architectural decisions made so far as well as open questions. It is not exhaustive. Comments and feedback are appreciated. 

### Previous prototype
The first prototype of Karta was made with the Godot engine and its scripting language GDscript. Godot has a lot of features that made it simple to get started and get something working quickly. It worked well, but with its shaky foundation, the prototype quickly became unwieldy and started breaking at odd places, and I didn't have the knowledge or the tools to debug it. Spaghetti. But the experience was encouraging, so I started to look at my options for continuing the experiment.

### Rust
I landed on Rust after weighing it against C++ and deciding that I didn't want to have to deal with manual memory management and runtime errors. I want clear feedback up front on whether my code will work or not. Godot's deep inheritance trees also left a sour impression on me, so I was further lulled in to Rust by its composition features. 

### Bevy
From Godot I was left with an optimistic view of using game engines for native applications. I tried Bevy, using it to load a folder of files as circles in a 2D space. The ECS system clicked, and I imagined it would be a very intuitive and ergonomic way to manage nodes and edges in Karta. The ECS has the downside of handwaving away some of Rusts safety guarantees, but so far the tradeoff has been completely worth it. Bevy's community has also proven highly helpful and knowledgeable. Karta will be able to leverage future improvements to Bevy. 

### Karta's dependencies

### Module and Plugin structure

### Open questions
Some things I'm not currently dealing with but am imagining will be important in the future. Refer to VISION.md for the features that will want these technical challenges addressed. 

* Indexing of nodes and connections using a database (SurrealDB being the frontrunner)
* How to tackle the serialization of non-unicode paths, to support moving of a vault from different operating systems?
* How will the evaluation of the node graph for real-time composition work, and how will it be made performant?
* How would the exporting of specific networks work? How to minimize the size of the exports and runtimes? 
* How to export a runtime to the web?
* How should plugin and custom node support be implemented? 

