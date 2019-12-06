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

    pub fn shortest_path(&mut self, start: &Coordinates, goal: &Coordinates, current_range_in_meters: u32, max_range_in_meters: u32) -> Result<Route, &str> {
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
                let route = self.backtrack_path(start_index, node.index, current_range_in_meters, max_range_in_meters);
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

    pub fn nearest_charging_station(&self, coords: &Coordinates) -> Coordinates {
        let mut dist = u32::max_value();
        let mut temp_dist = 0;
        let mut chosen_coords = coords;
        let required_charging = ChargingOptions::from(self.mode);
        debug!("Current coordinate {:?}", &coords);
        // TODO: extend graph with charging node set, then search nearest neighbor for node if no way available
        for charging_node in &self.graph.charging_nodes {
            if charging_node.charging_options.contains(required_charging) {
                temp_dist = coords.distance(&charging_node.coordinates);
                if temp_dist < dist {
                    chosen_coords = &charging_node.coordinates;
                    dist = temp_dist
                }
            }
        }
        debug!("Found charging station with coordinates {:?}", &chosen_coords);
        chosen_coords.clone()
    }

    fn backtrack_path(&self, start_index: usize, goal_index: usize, mut current_range_in_meters: u32, max_range_in_meters: u32) -> Route {
        let mut path = Vec::new();
        let mut time = 0;
        let mut distance = 0;
        let mut temp_distance = 0;
        let mut edge = self.prev[goal_index];

        loop {
            distance += edge.distance;
            temp_distance += edge.distance;
            time += edge.time(self.mode);

            if temp_distance > current_range_in_meters {
                let charging_station_coords = self.nearest_charging_station(&self.graph.coordinates(edge.target_index).clone());
                let closest_to_charging_index = self.graph.nearest_neighbor(&charging_station_coords, self.mode);
                // reset distance
                temp_distance = 0;
                // we recharged
                current_range_in_meters = max_range_in_meters;
                // TODO: find path from current node to nearest charging station, and find path from charging station to goal
            }
            path.push(self.graph.coordinates(edge.target_index).clone());

            edge = self.prev[edge.source_index];
            if edge.source_index == start_index {
                break;
            }
        }
        path.push(self.graph.coordinates(edge.source_index).clone());
        Route::new(path, time, distance)
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
}

impl Route {
    pub fn new(path: Vec<Coordinates>, time: u32, distance: u32) -> Self {
        Self {
            path,
            time,
            distance,
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
        let graph = Graph::from_bin("stuttgart-regbez-latest.bin");
        let mut router = Router::new(&graph, Car, Distance);
        let start =
            Coordinates::from(Point::new(48.74504025447951, 9.108545780181887));
        let goal =
            Coordinates::from(Point::new(48.74465821861257, 9.107344150543215));
        let max_distance = start.distance(&goal) * 2;

        let route = router.shortest_path(&start, &goal, 300, 350);
        assert!(route.unwrap().distance < max_distance);
    }

    #[test]
    fn time_stuttgart_hamburg() {
        let graph = Graph::from_bin("germany-latest.bin");
        let mut router = Router::new(&graph, Car, Time);
        let stuttgart = Coordinates::from(Point::new(48.783418, 9.181945));
        let hamburg = Coordinates::from(Point::new(53.552483, 10.006797));
        let now = Instant::now();
        let route = router.shortest_path(&stuttgart, &hamburg, 300, 500);
        let secs = now.elapsed().as_secs();
        assert!(route.is_ok());
        assert!(secs < 10);
    }
}
