use std::cmp::Ordering;
use std::collections::BinaryHeap;

use stable_vec::StableVec;

use log::debug;

use crate::graph::{Edge, Graph};
use crate::osm::Coordinates;
use crate::osm::options::{Routing, Transport, ChargingOptions};
use crate::osm::options::Routing::Time;
use crate::osm::options::Transport::Car;

pub struct Router<'a> {
    graph: &'a Graph,
    mode: Transport,
    routing: Routing,

    queue: BinaryHeap<RouterNode>,
    cost: Vec<u32>,
    prev: StableVec<&'a Edge>,
}

impl<'a> Router<'a> {
    pub fn new(graph: &'a Graph, mode: Transport, routing: Routing) -> Self {
        Self {
            graph,
            mode,
            routing,

            queue: BinaryHeap::with_capacity(graph.nodes.len()),
            cost: vec![u32::max_value(); graph.nodes.len()],
            prev: StableVec::with_capacity(graph.nodes.len()),
        }
    }

    pub fn shortest_path(&mut self, start: &Coordinates, goal: &Coordinates) -> Result<Route, &str> {
        let start_index = self.graph.nearest_neighbor(start, self.mode)?;
        let start_id = self.graph.node(start_index).id;
        let goal_index = self.graph.nearest_neighbor(goal, self.mode)?;
        let goal_id = self.graph.node(goal_index).id;
        if start_id == goal_id {
            return Err("No path found, start is goal");
        }

        self.cost[start_index] = 0;
        self.queue.push(RouterNode::new(start_index, 0, 0));
        while let Some(node) = self.queue.pop() {
            let id = self.graph.node(node.index).id;
            if id == goal_id {
                let route = self.backtrack_path(start_index, node.index);
                debug!("Distance of calculated route is {}.", &route.distance);
                return Ok(route);
            }
            // better solution already found
            if node.cost > self.cost[node.index] {
                continue;
            }

            for edge in self.graph.edges(node.index) {
                if !edge.transport.contains(self.mode) {
                    continue;
                }

                let cost = node.cost + edge.cost(self.mode, self.routing);
                if cost < self.cost[edge.target_index] {
                    let heuristic = self.heuristic(edge.target_index, goal_index);
                    let next = RouterNode::new(edge.target_index, cost, heuristic);
                    self.prev.insert(next.index, edge);
                    self.cost[next.index] = next.cost;
                    self.queue.push(next);
                }
            }
        }
        Err("No path found")
    }

    pub fn calc_route_with_charging_station(&mut self, actual_start: &Coordinates, actual_goal: &Coordinates, current_range: &u32) -> Result<Route, &str> {
        let coords_of_chosen_charging =
            self.get_optimal_charging_station_coords(actual_start, actual_goal, current_range.clone());
        let nearest_neighbor = self.graph.nearest_neighbor(&coords_of_chosen_charging, self.mode)?;
        let nearest_neighbor_coords = &self.graph.node(nearest_neighbor).coordinates;
        let route = self.shortest_path(actual_start, nearest_neighbor_coords);
        route
    }

    pub fn get_optimal_charging_station_coords(&self, actual_start: &Coordinates, actual_goal: &Coordinates, current_range: u32) -> Coordinates {
        let mut global_dist_from_start = 0;
        let mut global_dist_to_goal = u32::max_value();
        let mut charging_coords = actual_start;
        let required_charging = ChargingOptions::from(self.mode);

        for charging_node in &self.graph.charging_nodes {
            if charging_node.charging_options.contains(required_charging) {
                let dist_from_start = actual_start.distance(&charging_node.coordinates);
                let dist_to_goal = actual_goal.distance(&charging_node.coordinates);
                // add 1,5 as treshold since calculated distance is not the actual distance when driven but linear distance
                if f64::from(dist_from_start) * 1.5 < f64::from(current_range) && dist_from_start > global_dist_from_start
                    && dist_to_goal < global_dist_to_goal {
                    global_dist_from_start = dist_from_start;
                    global_dist_to_goal = dist_to_goal;
                    charging_coords = &charging_node.coordinates;
                }
            }
        }
        charging_coords.clone()
    }

    fn backtrack_path(&mut self, start_index: usize, goal_index: usize) -> Route {
        let mut path = Vec::new();
        let mut time = 0;
        let mut distance = 0;
        let mut edge = self.prev[goal_index];

        loop {
            distance += edge.distance;
            time += edge.time(self.mode);

            path.push(self.graph.coordinates(edge.target_index).clone());

            edge = self.prev[edge.source_index];
            if edge.source_index == start_index {
                break;
            }
        }
        path.push(self.graph.coordinates(edge.source_index).clone());
        Route::new(path, time, distance, None)
    }

    fn heuristic(&self, from: usize, to: usize) -> u32 {
        if self.mode == Car && self.routing == Time {
            0
        } else {
            self.graph.coordinates(from)
                .distance(self.graph.coordinates(to))
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct RouterNode {
    index: usize,
    cost: u32,
    heuristic: u32,
}

impl RouterNode {
    fn new(index: usize, cost: u32, heuristic: u32) -> Self {
        Self {
            index,
            cost,
            heuristic,
        }
    }

    fn priority(&self) -> u32 {
        self.cost + self.heuristic
    }
}

impl Ord for RouterNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority().cmp(&self.priority())
    }
}

impl PartialOrd for RouterNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct Route {
    pub path: Vec<Coordinates>,
    pub time: u32,
    pub distance: u32,
    pub visited_charging: Option<Vec<Coordinates>>,
}

impl Route {
    pub fn new(path: Vec<Coordinates>, time: u32, distance: u32, visited_charging: Option<Vec<Coordinates>>) -> Self {
        Self {
            path,
            time,
            distance,
            visited_charging,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;
    use std::time::Instant;

    use geo::Point;

    use crate::graph::Graph;
    use crate::graph::router::{Router, RouterNode};
    use crate::osm::Coordinates;
    use crate::osm::options::Routing::{Distance, Time};
    use crate::osm::options::Transport::Car;

    #[test]
    fn min_priority_queue() {
        let mut queue = BinaryHeap::with_capacity(5);
        queue.push(RouterNode::new(3, 3, 0));
        queue.push(RouterNode::new(1, 1, 0));
        queue.push(RouterNode::new(20, 20, 0));
        queue.push(RouterNode::new(2, 2, 0));
        queue.push(RouterNode::new(5, 5, 0));

        assert_eq!(queue.pop().unwrap().cost, 1);
        assert_eq!(queue.pop().unwrap().cost, 2);
        assert_eq!(queue.pop().unwrap().cost, 3);
        assert_eq!(queue.pop().unwrap().cost, 5);
        queue.push(RouterNode::new(15, 15, 0));
        assert_eq!(queue.pop().unwrap().cost, 15);
        assert_eq!(queue.pop().unwrap().cost, 20);
    }

    #[test]
    fn shortest_path() {
        let graph = Graph::from_bin("target/stuttgart-regbez-latest.bin");
        let mut router = Router::new(&graph, Car, Distance);
        let start =
            Coordinates::from(Point::new(48.7417761, 9.1036340));
        let goal =
            Coordinates::from(Point::new(48.7452193, 9.1025545));
        let max_distance = start.distance(&goal) * 2;

        let route = router.shortest_path(&start, &goal);
        let lol = route.unwrap();
        assert!(lol.distance < max_distance);
    }

    #[test]
    fn time_stuttgart_hamburg() {
        let graph = Graph::from_bin("germany-latest.bin");
        let mut router = Router::new(&graph, Car, Time);
        let stuttgart = Coordinates::from(Point::new(48.783418, 9.181945));
        let hamburg = Coordinates::from(Point::new(53.552483, 10.006797));
        let now = Instant::now();
        let route = router.shortest_path(&stuttgart, &hamburg);
        let secs = now.elapsed().as_secs();
        assert!(route.is_ok());
        assert!(secs < 10);
    }
}
