use std::str::FromStr;

use osmpbfreader::Way;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Highway {
    Motorway,
    Trunk,
    Primary,
    Secondary,
    Tertiary,
    Unclassified,
    Residential,

    MotorwayLink,
    TrunkLink,
    PrimaryLink,
    SecondaryLink,
    TertiaryLink,

    LivingStreet,
    Service,
    Track,
    Road,

    Cycleway,
}

impl Highway {
    pub fn from(way: &Way) -> Option<Self> {
        let tag = way.tags.get("highway")?;
        Self::from_str(tag).ok()
    }

    pub fn default_speed(self) -> Option<Kmh> {
        let speed = match self {
            Self::Motorway => 120,
            Self::Trunk => 120,
            Self::Primary => 100,
            Self::Secondary => 100,
            Self::Tertiary => 100,
            Self::Unclassified => 50,
            Self::Residential => 30,
            Self::MotorwayLink => 60,
            Self::TrunkLink => 60,
            Self::PrimaryLink => 50,
            Self::SecondaryLink => 50,
            Self::TertiaryLink => 50,
            Self::LivingStreet => 5,
            Self::Service => 30,
            _ => 30
        };
        Some(Kmh::new(speed))
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Kmh {
    pub speed: u32
}

impl Kmh {
    pub fn new(speed: u32) -> Self {
        Self { speed }
    }

    pub fn from(way: &Way) -> Option<Self> {
        let tag = way.tags.get("maxspeed")?;

        if let Ok(speed) = tag.parse::<u32>() {
            Some(Self::new(speed))
        } else {
            let speed: Vec<&str> = tag.split(' ').collect();
            if *speed.get(1)? == "mph" {
                let mph = speed.get(0)?
                    .parse::<u32>().ok()?;
                let kmh = mph as f32 * 1.609_344;
                return Some(Self::new(kmh as u32));
            }
            None
        }
    }

    pub fn time(self, distance: u32) -> u32 {
        let ms = self.speed as f32 / 3.6;
        (distance as f32 / ms).round() as u32
    }
}

#[cfg(test)]
mod tests {
    use crate::osm::highway::Kmh;

    #[test]
    fn time() {
        assert_eq!(14, Kmh::new(50).time(200));
        assert_eq!(36, Kmh::new(20).time(200));
        assert_eq!(144, Kmh::new(5).time(200));
    }
}
