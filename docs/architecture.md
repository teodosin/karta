This document outlines some of the architectural decisions made so far as well as open questions. It is not exhaustive. Comments and feedback are appreciated. 

Note: This doc has not been updated yet since the formation of the monorepo. Things are in flux, but some of the decisions made here are still relevant. 

### Background

#### Previous prototype
The first prototype of Karta was made with the Godot engine and its scripting language GDscript. Godot has a lot of features that made it simple to get started and get something working quickly. It worked well, but with its shaky foundation, the prototype quickly became unwieldy and started breaking at odd places, and I didn't have the knowledge or the tools to debug it. Spaghetti. But the experience was encouraging, so I started to look at my options for continuing the experiment.

#### Rust
I landed on Rust after weighing it against C++ and deciding that I didn't want to have to deal with manual memory management and runtime errors. I want clear feedback up front on whether my code will work or not. Godot's deep inheritance trees also left a sour impression on me, so I was further lulled in to Rust by its composition features. 

#### Bevy
From Godot I was left with an optimistic view of using game engines for native applications. I tried Bevy, using it to load a folder of files as circles in a 2D space. The ECS system clicked, and I imagined it would be a very intuitive and ergonomic way to manage nodes and edges in Karta. The ECS has the downside of handwaving away some of Rusts safety guarantees, but so far the tradeoff has been completely worth it. Bevy's community has also proven highly helpful and knowledgeable. Karta will be able to leverage future improvements to Bevy. 

------------------------------------------

## Overview 

As of writing this, Karta is undergoing a refactoring to better prepare it for new features and use cases. It is being split into a few different crates and repositories for maintainability and extendability reasons. Easier to upgrade or swap out parts in case the project pivots. 

### Crate structure

#### Indexing - karta_server

The core of Karta and its concept relies on an efficient and effective indexing of files into a graph structure. The library for this should be decoupled from the main Karta application so that it could eventually be accessed and modified by other applications as well. The separation also allows for the db to be run on a local or cloud server in addition to the local database file. 

Preferably, the solution would fulfill these requirements:
* Scalable, O(1) lookup times always
* Local first
* Support for virtual nodes (nodes that don't reference a file or directory)
* Support for arbitrary attributes on nodes and edges

The crate developed for this purpose is [karta_server](https://github.com/teodosin/karta_server), which is essentially Karta's wrapper around [Agnesoft Graph Database](https://github.com/agnesoft/agdb). 

#### Bevy Plugin - bevy_karta_client

Since the main Karta application is developed using Bevy, there needs to be a connective layer between the user interface and graphical elements being coded in Bevy and the database backend. [bevy_karta_client](https://github.com/teodosin/bevy_karta_client) is essentially the high-level backend of Karta, responsible for communication with the database. 

bevy_karta_client loads in nodes and edges from the database and turns them into Entities in Bevy's ECS. 

#### User Interface - bevy_karta_ui

[bevy_karta_ui](https://github.com/teodosin/bevy_karta_ui).

### Open questions
Some things I'm not currently dealing with but am imagining will be important in the future. Refer to VISION.md for the features that will want these technical challenges addressed. 

* How to tackle the serialization of non-unicode paths, to support moving of a vault from different operating systems?
* How will the evaluation of the node graph for real-time composition work, and how will it be made performant?
* How would the exporting of specific networks work? How to minimize the size of the exports and runtimes? 
* How to export a runtime to the web?
* How should plugin and custom node support be implemented? 

