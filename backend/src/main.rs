use std::{env, process};
use std::path::Path;
use std::time::Instant;

use log::debug;

use crate::graph::Graph;

mod graph;
mod logger;
mod osm;
mod rest;

fn main() {
    logger::init().unwrap();
    let now = Instant::now();
    let graph = graph();
    debug!("Parsing the PBF file took {:?} seconds", now.elapsed().as_secs());
    rest::init(graph);
}

fn graph() -> Graph {
    let pbf_name = if let Some(arg) = env::args().nth(1) { arg } else {
        println!("Please provide a *.osm.pbf file as argument");
        process::exit(1);
    };
    let name_stub = pbf_name.split('.').collect::<Vec<&str>>()[0];
    let bin_name = format!("{}.bin", &name_stub);

    if Path::new(&bin_name).exists() {
        debug!("Found existing graph");
        Graph::from_bin(&bin_name)
    } else {
        debug!("No existing graph found, parsing...");
        let graph = Graph::from_pbf(&pbf_name);
        graph.save(&bin_name);
        graph
    }
}
