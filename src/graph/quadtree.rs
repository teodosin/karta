//
use bevy::{prelude::*, utils::HashMap};
use std::collections::HashSet;

// Initial code from Chat-GPT4.

// The AABB struct defines a rectangle in 2D space.
#[derive(Debug, Clone)]
struct AABB {
    x_min: f32,
    y_min: f32,
    x_max: f32,
    y_max: f32,
}

impl AABB {
    fn new(x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> Self {
        AABB { x_min, y_min, x_max, y_max }
    }

    fn contains(&self, point: &Vec2) -> bool {
        point.x >= self.x_min && point.x <= self.x_max && point.y >= self.y_min && point.y <= self.y_max
    }

    fn intersects(&self, other: &AABB) -> bool {
        self.x_min <= other.x_max && self.x_max >= other.x_min && self.y_min <= other.y_max && self.y_max >= other.y_min
    }
}

// The QuadTree struct itself
#[derive(Debug)]
struct QuadTree {
    boundary: AABB,
    capacity: usize,
    entities: HashSet<Entity>,
    divided: bool,
    // Children of this quadtree node
    north_west: Option<Box<QuadTree>>,
    north_east: Option<Box<QuadTree>>,
    south_west: Option<Box<QuadTree>>,
    south_east: Option<Box<QuadTree>>,
}

impl QuadTree {
    // Create a new QuadTree
    fn new(boundary: AABB, capacity: usize) -> Self {
        QuadTree {
            boundary,
            capacity,
            entities: HashSet::new(),
            divided: false,
            north_west: None,
            north_east: None,
            south_west: None,
            south_east: None,
        }
    }

    // Subdivide the QuadTree into four children
    fn subdivide(&mut self) {
        let x_mid = (self.boundary.x_min + self.boundary.x_max) / 2.0;
        let y_mid = (self.boundary.y_min + self.boundary.y_max) / 2.0;

        let nw = AABB::new(self.boundary.x_min, y_mid, x_mid, self.boundary.y_max);
        let ne = AABB::new(x_mid, y_mid, self.boundary.x_max, self.boundary.y_max);
        let sw = AABB::new(self.boundary.x_min, self.boundary.y_min, x_mid, y_mid);
        let se = AABB::new(x_mid, self.boundary.y_min, self.boundary.x_max, y_mid);

        self.north_west = Some(Box::new(QuadTree::new(nw, self.capacity)));
        self.north_east = Some(Box::new(QuadTree::new(ne, self.capacity)));
        self.south_west = Some(Box::new(QuadTree::new(sw, self.capacity)));
        self.south_east = Some(Box::new(QuadTree::new(se, self.capacity)));

        self.divided = true;
    }

    // Insert an entity into the QuadTree
    fn insert(&mut self, entity: Entity, point: Vec2) -> bool {
        // If the point is not within this QuadTree's boundary, return false
        if !self.boundary.contains(&point) {
            return false;
        }

        // If there is space in this QuadTree, add the entity here
        if self.entities.len() < self.capacity {
            self.entities.insert(entity);
            return true;
        }

        // Otherwise, subdivide and then insert the point into whichever child will contain it
        if !self.divided {
            self.subdivide();
        }

        if self.north_west.as_mut().unwrap().insert(entity, point)
            || self.north_east.as_mut().unwrap().insert(entity, point)
            || self.south_west.as_mut().unwrap().insert(entity, point)
            || self.south_east.as_mut().unwrap().insert(entity, point)
        {
            return true;
        }

        // If we cannot insert it for some reason, return false
        false
    }

    // TODO: More quadtree methods as needed, such as query, remove, etc.
}


// Define a struct for your chunks
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct ChunkCoords(isize, isize);

#[derive(Debug)]
struct Chunk {
    quadtree: QuadTree,
}

impl Chunk {
    fn new(boundary: AABB) -> Self {
        Chunk {
            quadtree: QuadTree::new(boundary, 4), // you can adjust the capacity as needed
        }
    }
}

// The WorldMap contains all the chunks
struct WorldMap {
    chunks: HashMap<ChunkCoords, Chunk>,
    chunk_size: f32, // the size of each chunk
}

impl WorldMap {
    fn new(chunk_size: f32) -> Self {
        WorldMap {
            chunks: HashMap::new(),
            chunk_size,
        }
    }

    // Method to get the chunk coordinates for a given point
    fn get_chunk_coords(&self, point: &Vec2) -> ChunkCoords {
        ChunkCoords(
            (point.x / self.chunk_size).floor() as isize,
            (point.y / self.chunk_size).floor() as isize,
        )
    }

    // Insert an entity into the appropriate chunk's quadtree
    fn insert_entity(&mut self, entity: Entity, position: Vec2) {
        let coords = self.get_chunk_coords(&position);
        let chunk = self.chunks.entry(coords.clone()).or_insert_with(|| {
            // If the chunk does not exist, create it
            let chunk_boundary = AABB {
                x_min: coords.0 as f32 * self.chunk_size,
                y_min: coords.1 as f32 * self.chunk_size,
                x_max: (coords.0 as f32 + 1.0) * self.chunk_size,
                y_max: (coords.1 as f32 + 1.0) * self.chunk_size,
            };
            Chunk::new(chunk_boundary)
        });

        // Insert the entity into the chunk's quadtree
        if !chunk.quadtree.insert(entity, position) {
            // Handle the case where the entity can't be inserted, if needed
        }
    }

    // TODO: Add methods to remove entities, query for entities in a region, etc.
}

// Rest of your QuadTree and AABB implementation...
