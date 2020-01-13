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
    /**
    Create new pbf object.
    *
    @param filename: name of pbf file
    *
    @return pbf object
    */
    pub fn new(filename: &'a str) -> Self {
        Self {
            filename,
            node_indices: HashMap::new(),
            number_nodes: 0,
        }
    }

    /**
    Read pbf file.
    *
    @param self: pbf object with filename of pbf file
    *
    @return graph based on pbf input
    */
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

    /**
    Parse all charging stations within a pbf file.
    *
    @param self: pbf object with filename of pbf file
    *
    @return list of all nodes which are charging stations
    */
    fn parse_charging_stations(&mut self) -> Vec<ChargingNode> {
        // read pbf file
        let mut pbf = read_pbf(self.filename);
        let mut charging_nodes = Vec::new();
        // iterate over all objects in pbf file
        for object in pbf.par_iter() {
            if let OsmObj::Node(osm_node) = object.unwrap() {
                // check if node is charging station
                if osm_node.tags.contains("amenity", "charging_station") {
                    let id = osm_node.id;
                    let coordinates = Coordinates::new(
                        osm_node.decimicro_lat,
                        osm_node.decimicro_lon,
                    );
                    // we assume initially that a charging station can charge cars and bikes
                    let mut charging_options = ChargingOptions::CarBike;
                    // check if charging options are further specified and set charging options accordingly
                    if osm_node.tags.contains("car", "yes") && osm_node.tags.contains("bicycle", "yes") {
                        charging_options = ChargingOptions::CarBike;
                    } else if osm_node.tags.contains("car", "yes") {
                        charging_options = ChargingOptions::Car;
                    } else if osm_node.tags.contains("bicycle", "yes") {
                        charging_options = ChargingOptions::Bike;
                    }
                    // create new charging node with coordinates, id and charging options
                    let charging_node = ChargingNode::new(id.0, coordinates, charging_options);
                    charging_nodes.push(charging_node);
                }
            }
        }
        charging_nodes
    }

    /**
    Parse ways of a pbf file.
    *
    @param self: pbf object with filename of pbf
    *
    @return list of edges
    */
    fn parse_ways(&mut self) -> Vec<Edge> {
        // read pbf based on input pbf filename
        let mut pbf = read_pbf(self.filename);
        let mut edges = Vec::new();

        // iterate over all objects in pbf file
        for object in pbf.par_iter() {
            // unwrap only ways
            if let OsmObj::Way(way) = object.unwrap() {
                // derive highway type if possible
                let highway = Highway::from(&way);
                if highway.is_none() {
                    continue;
                }
                // get transport, max speed, one way
                let transport = Transport::from(highway.unwrap());
                let max_speed = Kmh::from(&way)
                    .or_else(|| highway.unwrap().default_speed()).unwrap();
                let is_oneway = is_oneway(&way);

                // get all nodes of ways
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
                    // if not oneway, set up a reverse edge
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

    /**
    Parse nodes of a pbf file.
    *
    @param self: pbf object with filename of pbf file
    *
    @return list of parsed nodes
    */
    fn parse_nodes(&self) -> StableVec<Node> {
        // read pbf file
        let mut pbf = read_pbf(self.filename);
        // create vec of nodes with len of node indices from way parsing
        let mut nodes =
            StableVec::with_capacity(self.node_indices.len());
        // iterate over all objects
        for object in pbf.par_iter() {
            // unwrap only nodes
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

    /**
    Create graph with nodes, edges and charging station nodes.
    *
    @param self: pbf object
    @param nodes: list of all nodes
    @param edges: list of all edges
    @param charging_nodes: list of all nodes with charging station
    *
    @return graph object
    */
    fn create_graph(&self, nodes: StableVec<Node>, mut edges: Vec<Edge>, charging_nodes: Vec<ChargingNode>) -> Graph {
        let offsets_len = self.node_indices.len() + 1;
        // create offset vec
        let mut offsets = vec![0; offsets_len];

        for edge in &mut edges {
            // get source and target coordinates of each edge
            let source_coords = &nodes[edge.source_index].coordinates;
            let target_coords = &nodes[edge.target_index].coordinates;
            // calc distance of each edge
            edge.distance = source_coords.distance(target_coords);
            // increment offset
            offsets[edge.source_index + 1] += 1;
        }

        for i in 1..offsets.len() {
            offsets[i] += offsets[i - 1]
        }
        Graph::new(nodes, offsets, edges, charging_nodes)
    }

    /**
    Insert node id into node indices.
    *
    @param self: pbf object
    @param id: id of node
    */
    fn insert_node_id(&mut self, id: NodeId) {
        // if already exists, do nothing
        if self.node_indices.contains_key(&id) {
            return;
        }
        // insert node id into indices
        self.node_indices.insert(id, self.number_nodes);
        self.number_nodes += 1;
    }
}

/**
Read pbf file.
*
@param filename: name of pbf file
*
@return osm pbf reader object
*/
fn read_pbf(filename: &str) -> OsmPbfReader<File> {
    let path = std::path::Path::new(filename);
    let file = std::fs::File::open(&path).unwrap();
    OsmPbfReader::new(file)
}


