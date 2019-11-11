use std::hash::{Hash, Hasher};

use geo::algorithm::haversine_distance::HaversineDistance;
use geo::Point;
use osmpbfreader::Way;
use serde::{Deserialize, Serialize};

pub mod pbf;
pub mod highway;
pub mod options;

pub fn is_oneway(way: &Way) -> bool {
    let tag = way.tags.get("oneway");
    // not oneway assumed if not specified
    if tag.is_none() {
        return false;
    }
    tag.unwrap() == "yes"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    lat: i32,
    lon: i32,
}

impl Coordinates {
    pub fn new(lat: i32, lon: i32) -> Self {
        Self {
            lat,
            lon,
        }
    }

    pub fn from(point: Point<f64>) -> Self {
        Self {
            lat: (point.x() / 1e-7) as i32,
            lon: (point.y() / 1e-7) as i32,
        }
    }

    pub fn lat(&self) -> f64 {
        f64::from(self.lat) * 1e-7
    }

    pub fn lon(&self) -> f64 {
        f64::from(self.lon) * 1e-7
    }

    fn lat_rounded(&self) -> i32 {
        (self.lat() * 10.0).round() as i32
    }

    fn lon_rounded(&self) -> i32 {
        (self.lon() * 10.0).round() as i32
    }

    pub fn point(&self) -> Point<f64> {
        Point::new(
            self.lat(),
            self.lon(),
        )
    }

    pub fn distance(&self, other: &Self) -> u32 {
        let distance = self.point().haversine_distance(&other.point());
        distance.round() as u32
    }
}

impl Eq for Coordinates {}

impl PartialEq for Coordinates {
    fn eq(&self, other: &Self) -> bool {
        self.lat_rounded() == other.lat_rounded() &&
            self.lon_rounded() == other.lon_rounded()
    }
}

impl Hash for Coordinates {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lat_rounded().hash(state);
        self.lon_rounded().hash(state);
    }
}
