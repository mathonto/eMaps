use std::{env, process};
use std::path::Path;
use std::time::Instant;

use log::debug;

use crate::graph::Graph;

mod graph;
mod logger;
mod osm;
mod rest;

/**
Entry point.
*/
fn main() {
    // init logger
    logger::init().unwrap();
    let now = Instant::now();
    // init graph
    let graph = graph();
    debug!("Parsing the PBF file took {:?} seconds", now.elapsed().as_secs());
    // init rest api
    rest::init(graph);
}

/**
Create graph from argument.
*/
fn graph() -> Graph {
    // get name of pbf file to be parsed
    let pbf_name = if let Some(arg) = env::args().nth(1) { arg } else {
        println!("Please provide a *.osm.pbf file as argument");
        process::exit(1);
    };
    // get name without extension
    let name_stub = pbf_name.split('.').collect::<Vec<&str>>()[0];
    // binary filename with same name as pbf input filename
    let bin_name = format!("{}.bin", &name_stub);

    // check if binary file already exists
    if Path::new(&bin_name).exists() {
        debug!("Found existing graph");
        // create graph from binary file
        Graph::from_bin(&bin_name)
    } else {
        debug!("No existing graph found, parsing...");
        // create graph from pbf file
        let graph = Graph::from_pbf(&pbf_name);
        // save graph to binary file
        graph.save(&bin_name);
        graph
    }
}
