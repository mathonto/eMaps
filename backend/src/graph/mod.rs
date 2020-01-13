use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};

use log::debug;
use serde::{Deserialize, Serialize};
use stable_vec::StableVec;

use crate::osm::highway::Kmh;
use crate::osm::options::{Routing, Transport, ChargingOptions};
use crate::osm::options::Routing::Time;
use crate::osm::options::Transport::{Bike, Car};
use crate::osm::pbf::Pbf;
use crate::osm::Coordinates;

pub mod router;
mod grid;

pub type Cells = HashMap<Coordinates, Vec<usize>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    nodes: Vec<Node>,
    offsets: Vec<usize>,
    edges: Vec<Edge>,
    cells: Cells,
    charging_nodes: Vec<ChargingNode>,
}

impl Graph {
    /**
    Create new graph.
    *
    @param nodes: parsed nodes
    @param offsets:
    @param edges: parsed edges
    @param charging_nodes: charging station nodes
    *
    @return Self: a new graph
    */
    pub fn new(nodes: StableVec<Node>, offsets: Vec<usize>, edges: Vec<Edge>, charging_nodes: Vec<ChargingNode>) -> Self {
        // StableVec does not implement Serialize
        let mut vec = Vec::with_capacity(nodes.capacity());
        // add all nodes to vec
        for (_, node) in nodes {
            vec.push(node);
        }
        // create grid
        let cells = grid::create(&vec);
        // create and return graph object with all data
        Self {
            nodes: vec,
            edges,
            offsets,
            cells,
            charging_nodes,
        }
    }

    /**
    Read pbf file and create graph.
    *
    @param filename: name of the pbf file to be read
    *
    @return Self: a new graph
    */
    pub fn from_pbf(filename: &str) -> Self {
        Pbf::new(filename).read()
    }

    /**
    Read binary file and create graph.
    *
    @param filename: name of the binary file to be read
    *
    @return Self: a new graph
    */
    pub fn from_bin(filename: &str) -> Self {
        debug!("Reading graph from {}...", filename);
        // open specified file
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        // deserialize graph from bin file
        let graph: Self = bincode::deserialize_from(reader).unwrap();
        debug!("Read graph from {}...", filename);
        graph
    }

    /**
    Save graph parsed from pbf to bin file.
    *
    @param self: graph to be saved
    @param filename: name of file to be saved
    */
    pub fn save(&self, filename: &str) {
        debug!("Writing graph to {}...", filename);
        // create new file
        let mut bin = File::create(filename).unwrap();
        // serialize graph
        let encoded = bincode::serialize(self).unwrap();
        // write serialized graph to bin file
        bin.write_all(&encoded).unwrap();
        debug!("Wrote graph to {}", filename);
    }

    /**
    Get node of graph.
    *
    @param self: graph
    @param index: index of node to be returned
    *
    @return &Node: reference of node
    */
    pub fn node(&self, index: usize) -> &Node {
        &self.nodes[index]
    }

    /**
    Get coordinates of a node of the graph.
    *
    @param self: graph
    @param index: index of node
    *
    @return &Coordinates: reference of coordinates of node
    */
    pub fn coordinates(&self, index: usize) -> &Coordinates {
        &self.node(index).coordinates
    }

    /**
    Get edges of node of graph.
    *
    @param self: graph
    @param node_index: index of node
    *
    @return &[Edge]: reference of vec of edges of node
    */
    pub fn edges(&self, node_index: usize) -> &[Edge] {
        let start = self.offsets[node_index];
        let end = self.offsets[node_index + 1];
        &self.edges[start..end]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: i64,
    pub coordinates: Coordinates,
}

impl Node {
    /**
    Create new node of graph.
    *
    @param id: id of node
    @param coordinates: coordinates of node
    *
    @return Self: node
    */
    pub fn new(id: i64, coordinates: Coordinates) -> Self {
        Self {
            id,
            coordinates,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChargingNode {
    pub id: i64,
    pub coordinates: Coordinates,
    pub charging_options: ChargingOptions,
}

impl ChargingNode {
    /**
    Create new node of a charging station
    *
    @param id: id of charging station node
    @param coordinates: coordinates of charging station
    @charging_options: ChargingOptions object specifiying whether charging station is valid for e-bike/e-car/both
    *
    @return Self: charging station node
    */
    pub fn new(id: i64, coordinates: Coordinates, charging_options: ChargingOptions) -> Self {
        Self {
            id,
            coordinates,
            charging_options,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub source_index: usize,
    pub target_index: usize,
    pub transport: Transport,
    pub distance: u32,
    pub max_speed: Kmh,
}

impl Edge {
    /**
    Create new edge of graph.
    *
    @param source_index: index of starting node
    @param target_index: index of target node
    @param transport: Transport object specifying allowed transportation mode on edge (bike, car)
    @param distance: distance of edge
    @max_speed: allowed max speed on edge
    *
    @return Self: new edge
    */
    pub fn new(source_index: usize, target_index: usize, transport: Transport,
               distance: u32, max_speed: Kmh) -> Self {
        Self {
            source_index,
            target_index,
            transport,
            distance,
            max_speed,
        }
    }
    /**
    Cost function of an edge.
    *
    @param self: edge
    @param mode: transportation mode on edge (car or bike/both)
    @routing: routing for distance or time
    *
    @return u32: cost of edge
    */
    pub fn cost(&self, mode: Transport, routing: Routing) -> u32 {
        // if routing mode is car and routing is for time, cost of edge is time needed for distance
        if mode == Car && routing == Time {
            self.max_speed.time(self.distance)
        } else {
            // Bike and Walk are assumed to have constant speed
            self.distance
        }
    }

    /**
    Time function of an edge
    *
    @param self: edge
    @param mode: transportation mode (bike or car)
    *
    @return u32: time needed to travel along distance of edge with max speed
    */
    pub fn time(&self, mode: Transport) -> u32 {
        match mode {
            // calc time with max speed of car on edge
            Car => self.max_speed.time(self.distance),
            // for bike assume constant speed of 20 kmh
            Bike => Kmh::new(20).time(self.distance),
            _ => panic!("Unsupported transport mode")
        }
    }
}

impl Ord for Edge {
    /**
    Compare function to order edges.
    *
    @param self: first edge
    @param other: second edge
    *
    @return Ordering: ordering for those edges
    */
    fn cmp(&self, other: &Self) -> Ordering {
        self.source_index.cmp(&other.source_index)
            .then_with(|| self.target_index.cmp(&other.target_index))
    }
}

impl PartialOrd for Edge {
    /**
    Compare function for partial ordering of edges.
    *
    @param self: first edge
    @param other: second edge
    *
    @return Option<Ordering>:
    */
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::graph::Graph;

    #[test]
    fn parse_germany() {
        let now = Instant::now();
        let graph = Graph::from_pbf("germany-latest.osm.pbf");
        graph.save("germany-latest.bin");
        let mins = now.elapsed().as_secs() / 60;
        assert!(mins < 10);
    }
}
