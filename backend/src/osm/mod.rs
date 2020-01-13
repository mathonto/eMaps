use std::hash::{Hash, Hasher};

use geo::algorithm::haversine_distance::HaversineDistance;
use geo::Point;
use osmpbfreader::Way;
use serde::{Deserialize, Serialize};

pub mod pbf;
pub mod highway;
pub mod options;

/**
Check if a way in osm is only oneway.
*
@param way: osm way
*
@return true if way is oneway, false otherwise
*/
pub fn is_oneway(way: &Way) -> bool {
    // get oneway tag of osm way
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
    /**
    Create new coordinates object.
    *
    @param lat: latitude value
    @param lon: longitude value
    *
    @return coordinates
    */
    pub fn new(lat: i32, lon: i32) -> Self {
        Self {
            lat,
            lon,
        }
    }

    /**
    Create coordinates based on point.
    *
    @param point: f64 point value
    *
    @return coordinates
    */
    pub fn from(point: Point<f64>) -> Self {
        Self {
            lat: (point.x() / 1e-7) as i32,
            lon: (point.y() / 1e-7) as i32,
        }
    }

    /**
    Get latitude of coordinates.
    *
    @param self: coordinates
    *
    @return latitude of coordinates
    */
    pub fn lat(&self) -> f64 {
        f64::from(self.lat) * 1e-7
    }

    /**
    Get longitude of coordinates.
    *
    @param self: coordinates
    *
    @return longitude of coordinates
    */
    pub fn lon(&self) -> f64 {
        f64::from(self.lon) * 1e-7
    }

    /**
    Get rounded latitude of coordinates.
    *
    @param self: coordinates
    *
    @return rounded latitude of coordinates
    */
    fn lat_rounded(&self) -> i32 {
        (self.lat() * 10.0).round() as i32
    }

    /**
    Get rounded longitude of coordinates.
    *
    @param self: coordinates
    *
    @return rounded longitude of coordinates
    */
    fn lon_rounded(&self) -> i32 {
        (self.lon() * 10.0).round() as i32
    }

    /**
    Get point from coordinates.
    *
    @param self: coordinates
    *
    @return point of coordinates
    */
    pub fn point(&self) -> Point<f64> {
        Point::new(
            self.lat(),
            self.lon(),
        )
    }

    /**
    Calculate haversine distance between two coordinates.
    *
    @param self: coordinates a
    @param other: coordinates b
    *
    @return distance between coordinates a and b
    */
    pub fn distance(&self, other: &Self) -> u32 {
        let distance = self.point().haversine_distance(&other.point());
        distance.round() as u32
    }
}

impl Eq for Coordinates {}

impl PartialEq for Coordinates {
    /**
    Equality function for two coordinates.
    *
    @param self: coordinates a
    @param other: coordinates b
    *
    @return true if a equals b, false otherwise
    */
    fn eq(&self, other: &Self) -> bool {
        self.lat_rounded() == other.lat_rounded() &&
            self.lon_rounded() == other.lon_rounded()
    }
}

impl Hash for Coordinates {
    /**
    Hash function for coordinates.
    *
    @param self: coordinates
    @param state: hasher
    */
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lat_rounded().hash(state);
        self.lon_rounded().hash(state);
    }
}
