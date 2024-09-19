// Important feature to allow for subgraph export and bundling with applications.

// GraphScope would be a pattern (unsure if struct or trait or something else) that
// will be needed to validate the scope of the graph for applications that use a subset
// of the graph. For example, the data in a game could be indexed within a larger Karta graph 
// during development, to keep track of how all of its elements relate to not only each other 
// but also to the data beyond the game. When exporting a release version, the data for the game
// could be extracted from the graph and stored in a new graph with a new root directory.
// To accomplish this, there needs to be a process in place to ensure that the data used in the 
// game during development gets properly exported and bundled with the game.

// This is a complex problem. My initial hunch is that the game should run its own instance of 
// fs_graph with the same access to the db as the development environment. The game would then
// have a Scope defined which restricts its access to only nodes that match a certain pattern. That 
// way it can't end up relying on data that exists in the large graph but not in the extracted
// subgraph. 

// Another way to validate subgraph completeness would be to simply expect the user application to 
// have tests that validate the data. I'm not sure if this would be enough to prevent the
// aforementioned reliance on data that exists in the large graph but not in the subgraph.

// For this feature it would also be important to allow exporting a subgraph and package it into
// WASM as well. Imagine packaging up a collection of knowledge for and publishing it as a webapp. 