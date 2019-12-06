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
use crate::osm::options::Transport::{Bike, Car, Walk};
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
    pub fn new(nodes: StableVec<Node>, offsets: Vec<usize>, edges: Vec<Edge>, charging_nodes: Vec<ChargingNode>) -> Self {
        // StableVec does not implement Serialize
        let mut vec = Vec::with_capacity(nodes.capacity());
        for (_, node) in nodes {
            vec.push(node);
        }
        let cells = grid::create(&vec);
        Self {
            nodes: vec,
            edges,
            offsets,
            cells,
            charging_nodes,
        }
    }

    pub fn from_pbf(filename: &str) -> Self {
        Pbf::new(filename).read()
    }

    pub fn from_bin(filename: &str) -> Self {
        debug!("Reading graph from {}...", filename);
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let graph: Self = bincode::deserialize_from(reader).unwrap();
        debug!("Read graph from {}...", filename);
        graph
    }

    pub fn save(&self, filename: &str) {
        debug!("Writing graph to {}...", filename);
        let mut bin = File::create(filename).unwrap();
        let encoded = bincode::serialize(self).unwrap();
        bin.write_all(&encoded).unwrap();
        debug!("Wrote graph to {}", filename);
    }

    pub fn node(&self, index: usize) -> &Node {
        &self.nodes[index]
    }

    pub fn coordinates(&self, index: usize) -> &Coordinates {
        &self.node(index).coordinates
    }

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

    pub fn cost(&self, mode: Transport, routing: Routing) -> u32 {
        if mode == Car && routing == Time {
            self.max_speed.time(self.distance)
        } else {
            // Bike and Walk are assumed to have constant speed
            self.distance
        }
    }

    pub fn time(&self, mode: Transport) -> u32 {
        match mode {
            Car => self.max_speed.time(self.distance),
            Bike => Kmh::new(20).time(self.distance),
            Walk => Kmh::new(5).time(self.distance),
            _ => panic!("Unsupported transport mode")
        }
    }
}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.source_index.cmp(&other.source_index)
            .then_with(|| self.target_index.cmp(&other.target_index))
    }
}

impl PartialOrd for Edge {
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
