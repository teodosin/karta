---
title: "The Vision Behind Karta"
date: 2025-08-03
status: draft
description: "Exploring the motivation and long-term vision for Karta - from file organization to creative authoring"
tags: ["vision", "creative-tools", "graph-based"]
---

# The Vision Behind Karta

This text aims to paint a picture of where Karta came from and where it might be going. It's a living document, intended to be updated as the project progresses and its inspirations and goals are clarified. Some sections rely on possibly vague metaphors that may or may not make sense to you. Feedback is welcome and much appreciated. 

Karta is currently being worked on as my master's thesis project. The topics covered there will likely be similar to this document, though obviously more in depth. It will be available to read when finished. 

Contents
* Motivation
* Introduction
* Design Goals, Principles, Inspirations
* Planned Features
* Potential Use-Cases 

## Motivation

Imagine you were working on a painting. The entire canvas, the whole possible space the creative work may occupy, is visible to you at once. As you work, you may choose to focus on composition and proportions, the big picture, or you may look closer and flesh out the details in a specific area, on a specific element. Working on zoomed in sections immediately updates the big picture, as they are part of the same unified object. Zooming out and looking at the entire thing reveals the pattern of relationships between the different parts of the painting, all the rhythms of colored shapes you have so far made. Your attention can flow smoothly through all the areas in the painting on all scales. By looking at what is there and what the entire space is, you may realize what is still missing. The process is thus naturally guided toward completion. 

Now imagine your ability to zoom out was taken away. You may see only 0.1% of the painting at a time. You have to move through it many times to build an adequate idea of how the pieces fit together. What if then you need to communicate the big picture to someone else? Or what if you don't work on it for a while and you forget? You have to take a different piece of paper and start drawing a map. When you make any change to the painting, you have to take the time to update the map. You also have to make sure that if someone else works on the painting, the maps are kept in sync. Repeat this for every quality that is interesting enough to be measured. I expect this to sound tedious to you, but that is how larger creative projects such as games, films and comics are managed. 

Creative projects are singular, cohesive entities. Our tools for creating and managing them generally do not reflect this. We store and edit the individual elements separately, composing them into scenes as needed, also separately. Yet the elements, the building blocks of creative works, always exist in relationship to one another. They repeat. They rhyme. They contrast, and have rhythm. The arrangement of elements in the finished work constitutes the experience of it, and this may demand careful design. But the tools used to do this design are isolated from the substance of the works. The maps have to be maintained separately, often on web-based platforms that can't access the project files anyway. This project started out as an attempt to address this disconnect. 

## Introduction

Karta, at its core, is a file browser with a node graph interface. 

It aims to be a tool to create a unified, continuous object out of disparate elements and therefore make the various relationships and design decisions between the elements visible and explicit. It should be an environment where managing these feels natural, tactile and engaging, much like arranging pieces in a physical space. Taking this a step further, it should also be an environment that facilitates the formation of new ideas and projects. Projects never exist in complete isolation when they are being made; they contain references to previous or other people's works, to notes jotted down weeks or years ago, and to stories from the real world. Many projects are started when a creative goes through or stumbles upon completely unrelated pieces of older work and sees a new connection. 

There are a few observations that led me to believe an app like this could be possible and worth making. I will quickly list what I think were the main ones. First was the recognition that all elements that a creative project may consist of can be digitized and are usually stored as files in a file system. The second was that all data, and therefore all files, could be represented as a node graph. Indivisible units of the file are nodes, and their relationships are edges. The relationships between the files themselves may also be represented as a graph. Third was the observation that node-based creative authoring tools (Houdini, TouchDesigner, Nuke, Notch, Blender etc.) are firmly established in the industry for their flexibility and power. 

Karta being a file browser naturally follows from this. There are several benefits to this approach. 

A file browser is inherently a starting point for any creative projects, so positioning Karta at this layer would make it a natural gateway to accessing and managing all elements of a creative work. There it would tap into the universality of file systems, not restricting users to specific file formats, making it broadly applicable and versatile. This also means that Karta would not be competing with existing behemoths of the creative industry. Instead it would act as an in-between, organizing and defining relationships between files from more specialized applications. 

The node graph interface could transform traditional file management workflows. It would allow users to visualize and organize their files not just by their positions in the hierarchy, but by their relationships and context within the project. This more closely mirrors the interconnected nature of creative work than do traditional folder structures. 

Placing the files in a free-form dynamic canvas and allowing arbitrary connections to be made between them lays the foundation for creative authoring. It would start as quick edits; new virtual node types could be introduced that don't just store data, but transform it. Image files could be connected to resize or crop nodes, and the result from that could be saved into a new image. A composite node could combine multiple images together. By adding just a few node types supporting operations on a few universal file types, there would be enough potential ways to use them to actually be useful. Then, by adding the ability to decompose individual files into constituent nodes *in the same graph*, I suspect, or rather hope, that the potential would snowball from there. 

## Design Goals and Inspiration

Many of these I briefly touched upon in the previous section. Here I will highlight them in some more detail, in no particular order. These include UX and technical/architectural goals. Karta should be:

#### Interoperable
Karta should store the metadata of connections, settings and positions in a human-readable format locally on the user's computer. Since it operates just on the file system, it should be easy to start and quit using. Eventually Karta could parse markdown files and turn wikilinks into node connections, making it compatible and complementary with Obsidian as well. 

#### Data-driven
Nodes and edges provide the atomic units from which virtually anything could be made, so they need to be performant and flexible. They should support arbitrary amounts and types of attributes that should then be easily accessible by other nodes or parts of the program. 

#### Continuous
Everything in a Karta vault should connect to the same graph. Navigating this graph should be visually continuous in all dimensions. This means that the user should be able to "walk" from any view to any other without (in my opinion) jarring instant transitions. This would include filtering of nodes, Of course, these animations and transitions should be optional. They might not be to every user's taste. My desire to do this is based on the intuition that smooth transitions could help the user keep track of where they are and where everything else is going, to enhance the tactility of the app. This relates to the earlier painting metaphor as well. These are easier to turn off than to add in later, so I will attempt to implement them, for the sake of the experiment. 

#### Visual, Externalizing 
A core principle of Karta is that it should allow explicitly defining and showing details that would otherwise be relegated to the user's mental model or text documents. Insights about the node network should be available to the user at a click and a glance. Node and file types should have (optional) visual previews right on the nodes themselves. 

#### Immediate
Immediate and tactile, engaging. Actions should have instant visual feedback. Moving and wiring together nodes should feel fun and intuitive. I want it to feel as much like physical crafts as possible. 

#### Aesthetic
#### Performative


#### Node-based
As many things as possible in the app should eventually be implemented using the same nodes and edges abstractions. Yes, that includes the UI. UI panels and menus should be nodes. For example, a node properties menu would also be a node. It would be allowed to exist in the pannable and zoomable graph view. But if all nodes gain the ability to be "pinned" to the camera's UI layer, the menu could easily be made to behave as floating menus usually do.  


## Key Features

#### Contexts
A giant mega-graph with all the files and nodes in a user's vault could be fun for a couple minutes, but is very unwieldy and impractical. It is a requirement that the user can intuitively limit their view to only what they care about. And if you recall the design goal of continuity, the user should be able to travel to any other view from their current one. Contexts accomplish this. Think of them like a node's local graph. Every node has a context, and the graph is always viewed from one. The context stores all of a node's connections as well as their relative positions inside that context. All nodes' positions are stored like this. Nodes do not have absolute positions in the graph, only positions relative to their connections. The context you are in determines the positions of nodes in the interface. 

#### Expansion and Collapsing
The user should be able to look deeper into the network without leaving their current context. Any connected nodes' own connections could be expanded into the currently visible graph. Any selection of nodes should be able to be collapsed into an alias node or hidden from view. The user should have precise control over the complexity of the network they are engaging with. 

#### Force Direction and Layouting
This is experimental. The idea is to use force simulations to layout the nodes in the viewport by default instead of the user doing it manually. You could have different forces affecting nodes of different depths. Having a force simulation is useful because the file system itself is treated as a network, so adding files to a folder outside of Karta would add those files to that node's context. With strictly manual positioning the user would have to reposition potentially dozens or hundreds of nodes when they come back to Karta. 

#### Pinning and Filters
Both of the previous features can be more finely controlled with selections, pins and filters. Any node can be pinned to be unaffected by the force simulation, or to remain in the viewport even after the user changes context. Or both, naturally. Filters are queries for the node network and can be used for a variety of purposes. They can be connected to forces to finetune the layouting simulation, or merely to hide or spawn some group of nodes. Filters can be set to look at the types of nodes and their attributes or the edges and their attributes. 

#### Previews and Composition
Nodes each contain a piece of data which can often be represented visually. Image previews are a no-brainer. Operators could also be in visualised right in the nodes, taking note from apps like TouchDesigner and Tooll3. Operator chains can be evaluated and cached. Real-time composition should also be feasible; set a node as the active output node, and then Karta displays its data or evaluates its inputs to produce an image and display it in either the graph background or a separate window. For example images would be displayed just as they are. Operator chains would perform real-time composition. 

#### File Decomposition 
Support should be implemented for creating interpreters for any file type so that it can be split up and loaded as a network of nodes. This network is expanded into the current context. Then they can be positioned and connected like any normal node. In the thesis I will attempt to create a rough interpreter for SVG's. 


#### Play, Perform, Package
With such a flexible way to visualize and edit the graph, it makes sense to allow for another type of interaction with it. In play mode, nodes would be buttons with actions associated with them. These actions could modify the active output, move us to different parts of the graph, control the camera, or even modify the graph itself. The ways to interact with the graph and the data in it could be precisely limited and designed for performance or installation purposes. In the future, with a separate runtime, specific networks and their files could be exported as standalone executables to be put on the web or run natively. 


## Challenges

#### Technical
#### Usability



## Potential Use-Cases 

With the foundation described above, Karta should be stable, powerful and versatile enough to be taken in a number of directions. Either the whole app could be specialized, or specialized functionality could be added with modules and/or plugins. These are ideas for where the app could go in the medium and long term, provided that I am able to work on it full or part-time and get help at some point. 

#### Performance and Presentation
In play mode, Karta could be used for linear presentations or for more freeform performances. The graph could be treated as a state machine, where clicking different nodes modifies the output and moves it to different states. 

#### Demo and Web Exports
In the future, with a separate runtime, specific networks and their files could be exported as standalone executables to be put on the web or run natively. It would be nice to be able to only package the nodes and features that are used by the exported network. I don't currently have an idea for how to go about implementing this, but I would like to not accrue too much tech debt before giving this some thought.  

#### Bevy Visualization
I am fascinated by the idea of using Karta as a visualizer and quick editor for Bevy. Entities and their relationships could be represented in the graph as nodes and edges. Systems, system sets and their ordering likewise. Same for events. Being able to flexibly visualise these while a game is running could be really powerful. Now imagine if the Rust files themselves could be parsed and decomposed into a node graph, showing the module and plugin structure of the project. Nodes could be compiled into Rust code. All this would be strictly complementary to code-first development and to the upcoming Bevy editor. Doing all this is well beyond my current skill level, so I don't want to give promises or plans. If you are a fellow Bevy user and this sounds interesting to you, please get in touch. I'd love to bounce some ideas around. 

#### Integrations
Plugins could be built for other applications that are able to read and edit Karta's context files. Context files don't themselves contain other files but merely their paths and relationships. This could be leveraged to, for example, automatically import files on startup into the other application, making the transition between that application and Karta much more seamless. 





