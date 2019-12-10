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
    let mut current_range_in_meters = &request.current_range.parse::<u32>().unwrap() * 1000;
    let max_range_in_meters = &request.max_range.parse::<u32>().unwrap() * 1000;
    debug!("Calculating path...");
    debug!("Current range of e-vehicle is {}meters", &current_range_in_meters);
    debug!("Max. range of e-vehicle is {}meters", &max_range_in_meters);
    let now = Instant::now();
    let route = router.shortest_path(
        &request.start.coordinates(),
        &request.goal.coordinates(),
    );

    let mut visited_charging_coords = Vec::new();
    let mut final_path = Vec::new();
    let mut final_distance = 0;
    let mut final_time = 0;

    match route {
        Ok(rt) => {
            if rt.distance > current_range_in_meters {
                let mut charging_router = Router::new(
                    state.get_ref(),
                    Transport::from_str(&request.transport).unwrap(),
                    Routing::from_str(&request.routing).unwrap(),
                );
                let charging_route = charging_router.calc_route_with_charging_station(&request.start.coordinates(),
                                                                                      &current_range_in_meters);
                current_range_in_meters = max_range_in_meters.clone();
                match charging_route {
                    Ok(mut charging_rt) => {
                        let first_visit = charging_router.get_optimal_charging_station_coords(&request.start.coordinates(), current_range_in_meters.clone());
                        visited_charging_coords.push(first_visit);

                        final_distance += charging_rt.distance;
                        final_time += charging_rt.time;
                        &charging_rt.path.remove(0);
                        let mut final_router = Router::new(
                            state.get_ref(),
                            Transport::from_str(&request.transport).unwrap(),
                            Routing::from_str(&request.routing).unwrap(),
                        );
                        let final_route = final_router.shortest_path(charging_rt.path.get(0).unwrap(), &request.goal.coordinates());
                        match final_route {
                            Ok(mut final_rt) => {
                                final_distance += final_rt.distance;
                                final_time += final_rt.time;
                                for entry in &final_rt.path {
                                    final_path.push(entry.clone());
                                }
                                for entry in &charging_rt.path {
                                    final_path.push(entry.clone());
                                }
                                let final_route = Route::new(final_path, final_time, final_distance, Option::from(visited_charging_coords));
                                Ok(HttpResponse::Ok().json(Response::from(&final_route)))
                            }
                            Err(final_err) => {
                                Err(Error(final_err.to_string()))
                            }
                        }
                    }
                    Err(error) => {
                        debug!("No charging path found, calculation took {}ms", now.elapsed().as_millis());
                        Err(Error(error.to_string()))
                    }
                }
            } else {
                debug!("Calculated path in {}ms", now.elapsed().as_millis());
                Ok(HttpResponse::Ok().json(Response::from(&rt)))
            }
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
    current_range: String,
    max_range: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    path: Vec<FloatCoordinates>,
    time: u32,
    distance: u32,
    visited_charging_coords: Vec<FloatCoordinates>,
}

impl Response {
    fn from(route: &Route) -> Self {
        let path = route.path.iter()
            .map(|coord| FloatCoordinates::from(coord))
            .collect();
        let charging = route.visited_charging.as_ref().unwrap();
        let visited_charging_coords = charging.iter()
            .map(|coord| FloatCoordinates::from(coord))
            .collect();
        Self {
            path,
            time: route.time,
            distance: route.distance,
            visited_charging_coords,
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
