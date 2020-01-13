use std::collections::HashMap;

use crate::graph::{Cells, Graph, Node};
use crate::osm::Coordinates;
use crate::osm::options::Transport;

/**
Create new grid with cells.
*
@param nodes: vec of nodes of graph
*
@return Cells: grid with cells containing coordinates and indices
*/
pub fn create(nodes: &[Node]) -> Cells {
    // create new hashmap with len of amount of nodes
    let mut cells: Cells = HashMap::with_capacity(nodes.len());

    // iterate over all nodes
    for (i, node) in nodes.iter().enumerate() {
        let coordinates = node.coordinates.clone();
        // if coordinates already exist
        if let Some(indices) = cells.get_mut(&coordinates) {
            // add index to indices
            indices.push(i);
        } else {
            // create new index, add to indices
            let mut indices = Vec::new();
            indices.push(i);
            // insert new cell with coordinates and indices
            cells.insert(coordinates, indices);
        }
    }
    cells
}

impl Graph {
    /**
    Get nearest neighbor in graph.
    *
    @param self: graph
    @param coords: coordinates for which nearest neighbor is searched
    @mode: transportation mode of routing
    *
    @return Result<usize, &str>
    */
    pub fn nearest_neighbor(&self, coords: &Coordinates, mode: Transport) -> Result<usize, &str> {
        // get exact cell with correct coordinates
        let exact_cell = self.cells.get(coords)
            .ok_or("Couldn't locate point on map")?;
        let mut best = self.closest(vec![exact_cell; 1], coords, mode);

        // check 10% of the cells at max
        let max_radius = self.cells.len() as f32 * 0.1;
        for radius in 1..max_radius as i32 {
            let adjacent_cells = self.adjacent_cells(coords, radius);
            let adjacent = self.closest(adjacent_cells, coords, mode);

            if best.index.is_none() || best.dist > adjacent.dist {
                best = adjacent;
            } else {
                break;
            }
        }
        best.index.ok_or("No point matching transportation found")
    }

    fn adjacent_cells(&self, coords: &Coordinates, radius: i32) -> Vec<&Vec<usize>> {
        let mut cells = Vec::with_capacity((radius * 8) as usize);

        for i in -radius..=radius {
            for j in -radius..=radius {
                if i.abs() != radius && j.abs() != radius {
                    // cells from previous radii (inner cells) are not considered
                    continue;
                }
                let mut key = coords.point();
                key.0.x += f64::from(i);
                key.0.y += f64::from(j);

                let cell = self.cells.get(&Coordinates::from(key));
                if cell.is_none() {
                    // cell is outside of pbf file
                    continue;
                }
                cells.push(cell.unwrap());
            }
        }
        cells
    }
    /**
    * Get closest neighbor in grid for coordinates and transportation mode.
    *
    @param self: graph
    @param cells: grid with cells
    @param coords: coordinates for which closest is searched
    @mode: transportation mode of routing
    *
    @return Neighbor: nearest neighbor in grid for certain coordinates
    */
    fn closest(&self, cells: Vec<&Vec<usize>>, coords: &Coordinates, mode: Transport) -> Neighbor {
        let mut closest = Neighbor::new();

        // iterate over all cells
        for cell in cells {
            // iterate over indices in a cell
            for i in cell {
                // iterate over edges of index and check if transportation mode of edges matches with specified mode
                let matches_mode = self.edges(*i).iter()
                    .any(|e| e.transport.contains(mode));
                if !matches_mode {
                    continue;
                }
                // calc dist between found coordinates and specified coordinates
                let dist = self.coordinates(*i).distance(coords);
                if dist < closest.dist {
                    // update closest dist and index if shorter than current closest dist
                    closest.dist = dist;
                    closest.index = Some(*i);
                }
            }
        }
        closest
    }
}

struct Neighbor {
    index: Option<usize>,
    dist: u32,
}

impl Neighbor {
    /**
    Create new neighbor.
    *
    @return Self: neighbor
    */
    fn new() -> Self {
        Self {
            index: None,
            dist: u32::max_value(),
        }
    }
}
