// This file is for brainstorming the aspirational ergonomics of defining node types.

// Requirements:
// All node types have a return type. If the node is wired to an input socket of another
// node, the return type must match the input type of the socket. 

// Some nodes have inputs. 

// Some nodes, which I am calling "operators", may need caches to prevent unnecessary 
// recomputation. The node always tries to return the cache first, but is able to 
// recompute the value if the cache is not available. It will somehow know if the
// cache is valid or not. 



// Outstanding questions:
// Is it safe to assume that all inputs to the function can be inputs to the node?
// I am feeling like yes, because if the node requires constants, it can just be 
// defined in the function itself.


// Function variant
// ----------------------------------------------------------------
// #[derive(OperatorOrNodeOrSomething)]
// fn ExampleNode (
//     input_two: f32, // Primitive types are easy enough. 
//     // All inputs to this socket must be coercable to the type of this socket.

//     input_one: Query<&Transform>, // What if the node wants to use queries as inputs?
//     // I could imagine a scenario where we would want a node that doesn't need direct 
//     // inputs, but instead operates on some specific set of entities.


// ) -> ReturnType // What is the output type of the node? 
// {

// }

// // Trait implementation variant
// // ----------------------------------------------------------------
// // The node is defined as a trait implementation. 
// // This potentially allows for the ability to also change the visible ui of the node
// // type, more easily than if the node was defined as a function. At least that is my
// // current intuition on that. 

// pub trait Node {
//     fn execute(&self.cache: T , input: &Input) -> Output;
// }