use std::fmt;
use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;

use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::{App, HttpResponse, HttpServer, ResponseError};
use actix_web::get;
use actix_web::middleware::Logger;
use actix_web::post;
use actix_web::Result;
use actix_web::web::{Data, Json};
use geo::Point;
use log::debug;
use serde::{Deserialize, Serialize};
use serde::export::Formatter;

use crate::graph::Graph;
use crate::graph::router::{Route, Router};
use crate::osm::Coordinates;
use crate::osm::options::Routing;
use crate::osm::options::Transport;

const ADDRESS: &str = "localhost:8000";
const CORS_ADDRESS: &str = "http://localhost:3000";
const PATH_INDEX: &str = "frontend/build/index.html";
const PATH_FILES: &str = "frontend/build/static";

pub fn init(graph: Graph) {
    let state = Data::new(graph);

    HttpServer::new(move ||
        App::new()
            .register_data(state.clone())
            .service(index)
            .service(Files::new("/static", PATH_FILES)
                .show_files_listing()
                .use_last_modified(true))
            .service(shortest_path)

            .wrap(Logger::default())
            .wrap(Cors::new()
                .allowed_origin(CORS_ADDRESS)
                .allowed_origin(&format!("http://{}", ADDRESS))))
        .bind(ADDRESS).unwrap()
        .run().unwrap();
}

#[get("/")]
pub fn index() -> Result<NamedFile> {
    Ok(NamedFile::open(Path::new(PATH_INDEX))?)
}

#[post("/shortest-path")]
fn shortest_path(state: Data<Graph>, request: Json<Request>) -> Result<HttpResponse, Error> {
    let mut router = Router::new(
        state.get_ref(),
        Transport::from_str(&request.transport).unwrap(),
        Routing::from_str(&request.routing).unwrap(),
    );
    debug!("Calculating path...");
    let now = Instant::now();
    let route = router.shortest_path(
        &request.start.coordinates(),
        &request.goal.coordinates(),
    );

    match route {
        Ok(rt) => {
            debug!("Calculated path in {}ms", now.elapsed().as_millis());
            Ok(HttpResponse::Ok().json(Response::from(&rt)))
        }
        Err(err) => {
            debug!("No path found, calculation took {}ms", now.elapsed().as_millis());
            Err(Error(err.to_string()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    start: FloatCoordinates,
    goal: FloatCoordinates,
    transport: String,
    routing: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    path: Vec<FloatCoordinates>,
    time: u32,
    distance: u32,
}

impl Response {
    fn from(route: &Route) -> Self {
        let path = route.path.iter()
            .map(|coord| FloatCoordinates::from(coord))
            .collect();
        Self {
            path,
            time: route.time,
            distance: route.distance,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FloatCoordinates {
    lat: f64,
    lon: f64,
}

impl FloatCoordinates {
    fn from(coordinates: &Coordinates) -> Self {
        Self {
            lat: coordinates.lat(),
            lon: coordinates.lon(),
        }
    }

    fn coordinates(&self) -> Coordinates {
        Coordinates::from(Point::new(self.lat, self.lon))
    }
}

#[derive(Debug)]
struct Error(String);

impl ResponseError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(&self.0)
    }
}
