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

use crate::graph::{Graph, ChargingNode};
use crate::graph::router::{Route, Router};
use crate::osm::Coordinates;
use crate::osm::options::Routing;
use crate::osm::options::Transport;

const ADDRESS: &str = "localhost:8000";
const CORS_ADDRESS: &str = "http://localhost:3000";
const PATH_INDEX: &str = "frontend/build/index.html";
const PATH_FILES: &str = "frontend/build/static";

/**
Initialize server.
@param graph: current graph
*/
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
            .service(charging_stations)

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

/**
Handle request for all charging stations.
*
@param state: current state
*/
#[get("/charging-stations")]
fn charging_stations(state: Data<Graph>) -> Result<HttpResponse> {
    debug!("Getting charging stations...");
    let all_charging_stations = Graph::get_charging_stations(state.get_ref());
    debug!("Found {} charging stations", all_charging_stations.len());
    let resp = ChargingResponse::from(all_charging_stations);
    Ok(HttpResponse::Ok().json(resp))
}

/**
Handle shortest path request.
*
@param state: current state
@param request: json request from frontend
*/
#[post("/shortest-path")]
fn shortest_path(state: Data<Graph>, request: Json<Request>) -> Result<HttpResponse, Error> {
    // new router object with transport (car/bike) and routing (time/distance)
    let mut router = Router::new(
        state.get_ref(),
        Transport::from_str(&request.transport).unwrap(),
        Routing::from_str(&request.routing).unwrap(),
    );
    // parse current range and max range
    let mut current_range_in_meters = &request.current_range.parse::<u32>().unwrap() * 1000;
    let max_range_in_meters = &request.max_range.parse::<u32>().unwrap() * 1000;
    debug!("Calculating path...");
    debug!("Current range of e-vehicle is {}meters", &current_range_in_meters);
    debug!("Max. range of e-vehicle is {}meters", &max_range_in_meters);
    let now = Instant::now();
    // start shortest path calculation
    let route = router.shortest_path(
        &request.start.coordinates(),
        &request.goal.coordinates(),
    );

    match route {
        Ok(rt) => {
            // required range to travel route
            let mut required_range = rt.distance;
            // init variables in case of charging required
            let mut final_path = Vec::new();
            let mut final_distance = 0;
            let mut final_time = 0;
            let mut visited_charging_coords = Vec::new();
            let mut start = &request.start.coordinates().clone();
            let goal = &request.goal.coordinates().clone();

            let mut iter_count = 0;
            // while required range is bigger than current range of vehicle, we need to re-calculate the final route
            while required_range > current_range_in_meters {
                let mut charging_router = Router::new(
                    state.get_ref(),
                    Transport::from_str(&request.transport).unwrap(),
                    Routing::from_str(&request.routing).unwrap(),
                );
                // calc route to a charging station from original start
                let route_to_charging = charging_router.calc_route_with_charging_station(start, goal, &current_range_in_meters);
                match route_to_charging {
                    Ok(mut rt_charging) => {
                        // coordinates of visited charging station
                        let charging_coords = charging_router.get_optimal_charging_station_coords(start, goal, current_range_in_meters.clone());
                        visited_charging_coords.push(charging_coords);
                        // set visited charging station as new start
                        start = visited_charging_coords.get(iter_count).unwrap();
                        final_distance += rt_charging.distance;
                        final_time += rt_charging.time;
                        // vehicle is charged, current range is max range now
                        current_range_in_meters = max_range_in_meters;
                        //remove duplicate
                        rt_charging.path.remove(0);
                        final_path.push(rt_charging.path.clone());

                        let mut goal_router = Router::new(
                            state.get_ref(),
                            Transport::from_str(&request.transport).unwrap(),
                            Routing::from_str(&request.routing).unwrap(),
                        );
                        // calc shortest path from visited charging station to original goal
                        let route_to_goal = goal_router.shortest_path(start, goal);
                        match route_to_goal {
                            Ok(rt_goal) => {
                                // if route to goal is within range, add to path
                                if rt_goal.distance <= current_range_in_meters {
                                    final_distance += rt_goal.distance;
                                    final_time += rt_goal.time;
                                    // add path to list of paths
                                    final_path.push(rt_goal.path.clone());
                                }
                                required_range = rt_goal.distance;
                                iter_count += 1;
                            }
                            Err(error) => {
                                debug!("No path found, calculation took {}ms", now.elapsed().as_millis());
                                return Err(Error(error.to_string()));
                            }
                        }
                    }
                    Err(error) => {
                        debug!("No path found, calculation took {}ms", now.elapsed().as_millis());
                        return Err(Error(error.to_string()));
                    }
                }
                if iter_count > 100 {
                    debug!("No path found, calculation took {}ms", now.elapsed().as_millis());
                    return Err(Error("Please enter reasonable ranges.".parse().unwrap()));
                }
            }
            // if a charging station was visited
            if visited_charging_coords.len() > 0 {
                let mut result_path = Vec::new();
                // reverse array since we need first path first..
                final_path.reverse();
                for path in final_path {
                    for entry in path {
                        // add all entries in all path to obtain final result path
                        result_path.push(entry);
                    }
                }
                // create new final route
                let route = Route::new(result_path, final_time, final_distance, Option::from(visited_charging_coords));
                debug!("Path found, calculation took {}ms", now.elapsed().as_millis());
                Ok(HttpResponse::Ok().json(Response::from(&route)))
            } else {
                let route = Route::new(rt.path, rt.time, rt.distance, None);
                debug!("Path found, calculation took {}ms", now.elapsed().as_millis());
                Ok(HttpResponse::Ok().json(Response::from(&route)))
            }
        }
        Err(error) => {
            debug!("No path found, calculation took {}ms", now.elapsed().as_millis());
            Err(Error(error.to_string()))
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
struct ChargingResponse {
    charging_coords: Vec<FloatCoordinates>
}

impl ChargingResponse {
    fn from(charging_nodes: &Vec<ChargingNode>) -> Self {
        let charging_coords = charging_nodes.iter()
            .map(|coord| FloatCoordinates::from(&coord.coordinates))
            .collect();
        Self {
            charging_coords
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    path: Vec<FloatCoordinates>,
    time: u32,
    distance: u32,
    visited_charging_coords: Vec<FloatCoordinates>,
}

impl Response {
    /**
    Create response from route.
    *
    @param route: calculated route
    *
    @return response to return to frontend
    */
    fn from(route: &Route) -> Self {
        // get path as list of float coordinates
        let path = route.path.iter()
            .map(|coord| FloatCoordinates::from(coord))
            .collect();
        // get visited charging station coordinates to highlight in frontend
        let visited_charging_coords = route.visited_charging.clone();
        if visited_charging_coords.is_some() {
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
        } else {
            Self {
                path,
                time: route.time,
                distance: route.distance,
                visited_charging_coords: vec![],
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FloatCoordinates {
    lat: f64,
    lon: f64,
}

impl FloatCoordinates {
    /**
    Create float coordinates object for frontend from coordinates.
    *
    @param coordinates: coordinates to convert
    *
    @return float coordinates object
    */
    fn from(coordinates: &Coordinates) -> Self {
        Self {
            lat: coordinates.lat(),
            lon: coordinates.lon(),
        }
    }

    /**
    Get coordinates from float coordinates.
    *
    @param self: float coordinates to be converted
    *
    @return coordinates object
    */
    fn coordinates(&self) -> Coordinates {
        Coordinates::from(Point::new(self.lat, self.lon))
    }
}

#[derive(Debug)]
struct Error(String);

impl ResponseError for Error {}

impl Display for Error {
    /**
    Format error.
    *
    @param self: error
    @param f: formatter
    *
    @return formatted error
    */
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(&self.0)
    }
}
