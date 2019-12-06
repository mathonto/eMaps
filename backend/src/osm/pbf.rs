use std::collections::HashMap;
use std::fs::File;

use log::debug;
use osmpbfreader::{NodeId, OsmObj, OsmPbfReader};
use stable_vec::StableVec;

use crate::graph::{Edge, Graph, Node, ChargingNode};
use crate::osm::{Coordinates, is_oneway};
use crate::osm::highway::{Highway, Kmh};
use crate::osm::options::{Transport, ChargingOptions};

pub struct Pbf<'a> {
    filename: &'a str,
    node_indices: HashMap<NodeId, usize>,
    number_nodes: usize,
}

impl<'a> Pbf<'a> {
    pub fn new(filename: &'a str) -> Self {
        Self {
            filename,
            node_indices: HashMap::new(),
            number_nodes: 0,
        }
    }

    pub fn read(&mut self) -> Graph {
        debug!("Parsing charging stations...");
        let charging_stations = self.parse_charging_stations();
        debug!("Parsed {} charging stations", charging_stations.len());
        debug!("Parsing edges...");
        let edges = self.parse_ways();
        debug!("Parsed {} edges", edges.len());
        debug!("Parsing nodes...");
        let nodes = self.parse_nodes();
        debug!("Parsed {} nodes", nodes.capacity());
        debug!("Creating graph...");
        self.create_graph(nodes, edges, charging_stations)
    }

    fn parse_charging_stations(&mut self) -> Vec<ChargingNode> {
        let mut pbf = read_pbf(self.filename);
        let mut charging_nodes = Vec::new();
        for object in pbf.par_iter() {
            if let OsmObj::Node(osm_node) = object.unwrap() {
                if osm_node.tags.contains("amenity", "charging_station") {
                    let id = osm_node.id;
                    let coordinates = Coordinates::new(
                        osm_node.decimicro_lat,
                        osm_node.decimicro_lon,
                    );
                    let mut charging_options = ChargingOptions::CarBike;
                    if osm_node.tags.contains("car", "yes") && osm_node.tags.contains("bicycle", "yes") {
                        charging_options = ChargingOptions::CarBike;
                    } else if osm_node.tags.contains("car", "yes") {
                        charging_options = ChargingOptions::Car;
                    } else if osm_node.tags.contains("bicycle", "yes") {
                        charging_options = ChargingOptions::Bike;
                    }
                    let charging_node = ChargingNode::new(id.0, coordinates, charging_options);
                    charging_nodes.push(charging_node);
                }
            }
        }
        charging_nodes
    }

    fn parse_ways(&mut self) -> Vec<Edge> {
        let mut pbf = read_pbf(self.filename);
        let mut edges = Vec::new();

        for object in pbf.par_iter() {
            if let OsmObj::Way(way) = object.unwrap() {
                let highway = Highway::from(&way);
                if highway.is_none() {
                    continue;
                }

                let transport = Transport::from(highway.unwrap());
                let max_speed = Kmh::from(&way)
                    .or_else(|| highway.unwrap().default_speed()).unwrap();
                let is_oneway = is_oneway(&way);

                self.insert_node_id(*way.nodes.get(0).unwrap());
                for i in 1..way.nodes.len() {
                    let source_id = *way.nodes.get(i - 1).unwrap();
                    let source_index = *self.node_indices.get(&source_id).unwrap();
                    let target_id = *way.nodes.get(i).unwrap();
                    self.insert_node_id(target_id);
                    let target_index = *self.node_indices.get(&target_id).unwrap();

                    let edge = Edge::new(
                        source_index,
                        target_index,
                        transport,
                        0,
                        max_speed,
                    );
                    if !is_oneway {
                        let mut reverse = edge.clone();
                        reverse.source_index = target_index;
                        reverse.target_index = source_index;
                        edges.push(reverse);
                    }
                    edges.push(edge);
                }
            }
        }
        edges.sort();
        edges
    }

    fn parse_nodes(&self) -> StableVec<Node> {
        let mut pbf = read_pbf(self.filename);
        let mut nodes =
            StableVec::with_capacity(self.node_indices.len());
        for object in pbf.par_iter() {
            if let OsmObj::Node(osm_node) = object.unwrap() {
                let id = osm_node.id;
                if self.node_indices.contains_key(&id) {
                    let index = self.node_indices[&id];
                    let coordinates = Coordinates::new(
                        osm_node.decimicro_lat,
                        osm_node.decimicro_lon,
                    );
                    let node = Node::new(id.0, coordinates);

                    nodes.insert(index, node);
                }
            }
        }
        nodes
    }

    fn create_graph(&self, nodes: StableVec<Node>, mut edges: Vec<Edge>, charging_nodes: Vec<ChargingNode>) -> Graph {
        let offsets_len = self.node_indices.len() + 1;
        let mut offsets = vec![0; offsets_len];

        for edge in &mut edges {
            let source_coords = &nodes[edge.source_index].coordinates;
            let target_coords = &nodes[edge.target_index].coordinates;
            edge.distance = source_coords.distance(target_coords);

            offsets[edge.source_index + 1] += 1;
        }

        for i in 1..offsets.len() {
            offsets[i] += offsets[i - 1]
        }
        Graph::new(nodes, offsets, edges, charging_nodes)
    }

    fn insert_node_id(&mut self, id: NodeId) {
        if self.node_indices.contains_key(&id) {
            return;
        }
        self.node_indices.insert(id, self.number_nodes);
        self.number_nodes += 1;
    }
}

fn read_pbf(filename: &str) -> OsmPbfReader<File> {
    let path = std::path::Path::new(filename);
    let file = std::fs::File::open(&path).unwrap();
    OsmPbfReader::new(file)
}


