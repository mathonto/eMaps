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
    /**
    Create new router for routing.
    *
    @param graph: graph as base for routing
    @param mode: transportation mode (car/bike)
    @param routing: routing mode (distance/time)
    *
    @return Self: new route
    */
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

    /**
    Shortest path algorithm.
    *
    @param self: router
    @param start: start coordinates
    @param goal: goal coordinates
    *
    @return Result<Route, &str>: result object of shortest path routing
    */
    pub fn shortest_path(&mut self, start: &Coordinates, goal: &Coordinates) -> Result<Route, &str> {
        // retrieve start index based on nearest neighbor of start coordinates in graph
        let start_index = self.graph.nearest_neighbor(start, self.mode)?;
        let start_id = self.graph.node(start_index).id;
        // retrieve goal index based on nearest neighbor of goal coordinates in graph
        let goal_index = self.graph.nearest_neighbor(goal, self.mode)?;
        let goal_id = self.graph.node(goal_index).id;
        if start_id == goal_id {
            return Err("No path found, start is goal");
        }

        self.cost[start_index] = 0;
        // push start node to queue of router
        self.queue.push(RouterNode::new(start_index, 0, 0));
        // while still a node in the queue
        while let Some(node) = self.queue.pop() {
            // get id of current node in queue and check if equals goal id
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
            // iterate over edges of current node
            for edge in self.graph.edges(node.index) {
                // check if edge is valid for transportation method
                if !edge.transport.contains(self.mode) {
                    continue;
                }

                let cost = node.cost + edge.cost(self.mode, self.routing);
                if cost < self.cost[edge.target_index] {
                    let heuristic = self.heuristic(edge.target_index, goal_index);
                    // create new router node with current edge, cost and heuristic
                    let next = RouterNode::new(edge.target_index, cost, heuristic);
                    self.prev.insert(next.index, edge);
                    self.cost[next.index] = next.cost;
                    self.queue.push(next);
                }
            }
        }
        Err("No path found")
    }

    /**
    Shortest path calculation from original start to a charging station.
    *
    @param self: router
    @param actual_start: original start as chosen by user in frontend
    @param actual_goal: original goal as chosen by user in frontend
    @param current_range: current range of electric vehicle
    *
    @return calculated route from original start to charging station
    */
    pub fn calc_route_with_charging_station(&mut self, actual_start: &Coordinates, actual_goal: &Coordinates, current_range: &u32) -> Result<Route, &str> {
        // retrieve "optimal" charging station coordinates
        let coords_of_chosen_charging =
            self.get_optimal_charging_station_coords(actual_start, actual_goal, current_range.clone());
        // get nearest neighbor of charging station coordinates in graph
        let nearest_neighbor = self.graph.nearest_neighbor(&coords_of_chosen_charging, self.mode)?;
        // get coordinates of nearest neighbor as in graph
        let nearest_neighbor_coords = &self.graph.node(nearest_neighbor).coordinates;
        // calc shortest path from actual start to charging station
        let route = self.shortest_path(actual_start, nearest_neighbor_coords);
        route
    }

    /**
    Get optimal charging station coordinates.
    *
    @param self: router
    @param actual_start: original start as chosen by user in frontend
    @param actual_goal: original goal as chosen by user in frontend
    @param current_range: current range of electric vehicle
    *
    @return coordinates of charging station based on original start, goal, and current range
    */
    pub fn get_optimal_charging_station_coords(&self, actual_start: &Coordinates, actual_goal: &Coordinates, current_range: u32) -> Coordinates {
        let mut global_dist_from_start = 0;
        let mut global_dist_to_goal = u32::max_value();
        let mut charging_coords = actual_start;
        // get required charging station mode based on mode, e.g. for e-car or e-bike
        let required_charging = ChargingOptions::from(self.mode);

        // iterate over all charging stations
        for charging_node in &self.graph.charging_nodes {
            // check if charging station supports required charging mode
            if charging_node.charging_options.contains(required_charging) {
                let dist_from_start = actual_start.distance(&charging_node.coordinates);
                let dist_to_goal = actual_goal.distance(&charging_node.coordinates);
                /*
                add 1,5 as threshold since calculated distance is not the actual distance when driven but linear distance
                *
                check if charging station is reachable from original start based on current range
                and current range is used efficiently by choosing most distantly charging station
                reachable with current range and closest charging station to original goal
                */
                if f64::from(dist_from_start) * 1.5 < f64::from(current_range) && dist_from_start > global_dist_from_start
                    && dist_to_goal < global_dist_to_goal {
                    // update global comparison values and selected charging station coordinates
                    global_dist_from_start = dist_from_start;
                    global_dist_to_goal = dist_to_goal;
                    charging_coords = &charging_node.coordinates;
                }
            }
        }
        charging_coords.clone()
    }

    /**
    Shortest path backtracking.
    *
    @param self: router
    @param start_index: index of start node
    @param goal_index: index of goal node
    *
    @return final route for shortest path
    */
    fn backtrack_path(&mut self, start_index: usize, goal_index: usize) -> Route {
        let mut path = Vec::new();
        let mut time = 0;
        let mut distance = 0;
        let mut edge = self.prev[goal_index];

        loop {
            // increment distance and time based on edge
            distance += edge.distance;
            time += edge.time(self.mode);

            // add coordinates to path
            path.push(self.graph.coordinates(edge.target_index).clone());

            edge = self.prev[edge.source_index];
            if edge.source_index == start_index {
                break;
            }
        }
        path.push(self.graph.coordinates(edge.source_index).clone());
        Route::new(path, time, distance, None)
    }

    /**
    Heuristic for distance.
    *
    @param self: router
    @param from: start node for distance calculation
    @param to: target node for distance calculation
    *
    @return distance heuristic value
    */
    fn heuristic(&self, from: usize, to: usize) -> u32 {
        // if routing for time return 0
        if self.mode == Car && self.routing == Time {
            0
        } else {
            // calc (linear) distance from a to b
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
    /**
    Create new router node with index, cost and heuristic value
    *
    @param index: index of router node in graph
    @param cost: cost based on node + edge
    @param heuristic: value of distance heuristic
    *
    @return new router node
    */
    fn new(index: usize, cost: u32, heuristic: u32) -> Self {
        Self {
            index,
            cost,
            heuristic,
        }
    }

    /**
    Priority function for router nodes based on cost and heuristic
    *
    @param self: router node
    *
    @return priority value based on cost and heuristic
    */
    fn priority(&self) -> u32 {
        self.cost + self.heuristic
    }
}

impl Ord for RouterNode {
    /**
    Absolute ordering for router nodes
    *
    @param self: router node a
    @param other: router node b
    *
    @return ordering for router nodes a and b based on priority value
    */
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority().cmp(&self.priority())
    }
}

impl PartialOrd for RouterNode {
    /**
    Partial ordering for router nodes
    *
    @param self: router node a
    @param other: router node b
    *
    @return partial ordering for router nodes a and b based on priority value
    */
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
    /**
    Create new route.
    *
    @param path: path of shortest path
    @param time: time needed for route
    @param distance: distance of route
    @param visited_charging: (optional) list of visited charging stations on route
    *
    @return new route
    */
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
