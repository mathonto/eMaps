use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::osm::highway::Highway::{Cycleway, LivingStreet, Motorway, MotorwayLink,
                                   Primary, PrimaryLink, Residential, Road, Secondary,
                                   SecondaryLink, Service,
                                   Tertiary, TertiaryLink, Track, Trunk, TrunkLink, Unclassified};
use crate::osm::highway::Highway;
use crate::osm::options::Transport::{All, Bike, Car, CarBike};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Transport {
    Car,
    Bike,

    All,
    CarBike,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ChargingOptions {
    Car,
    Bike,
    CarBike,
    None,
}

impl ChargingOptions {
    pub fn from(transport: Transport) -> Self {
        match transport {
            Bike => ChargingOptions::Bike,
            Car => ChargingOptions::Car,
            CarBike => ChargingOptions::CarBike,
            _ => ChargingOptions::None
        }
    }

    pub fn contains(self, other: Self) -> bool {
        self == ChargingOptions::CarBike || self == other
            || self == ChargingOptions::CarBike && (other == ChargingOptions::Car || other == ChargingOptions::Bike)
    }
}

impl Transport {
    pub fn from(highway: Highway) -> Self {
        match highway {
            Residential | Tertiary | Unclassified | Service | LivingStreet | TertiaryLink => All,
            Secondary | SecondaryLink | Primary | PrimaryLink => CarBike,
            Motorway | MotorwayLink | Trunk | TrunkLink => Car,
            Track | Road | Cycleway => Bike
        }
    }

    pub fn contains(self, other: Self) -> bool {
        self == All || self == other ||
            (self == CarBike && (other == Car || other == Bike))
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Routing {
    Time,
    Distance,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::osm::highway::Highway::{Cycleway, LivingStreet, Motorway, MotorwayLink,
                                       Primary, PrimaryLink, Residential, Road, Secondary,
                                       SecondaryLink, Service,
                                       Tertiary, TertiaryLink, Track, Trunk, TrunkLink, Unclassified};
    use crate::osm::highway::Highway;

    #[test]
    fn transport_mapping() {
        let mut car: HashSet<Highway> = [
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
        ].iter().cloned().collect();
        let mut bike: HashSet<Highway> = [
            Primary,
            Secondary,
            Tertiary,
            Unclassified,
            Residential,
            PrimaryLink,
            SecondaryLink,
            TertiaryLink,
            LivingStreet,
            Service,
            Cycleway,
            Track,
            Road
        ].iter().cloned().collect();

        let all: HashSet<Highway> = car.intersection(&bike).cloned().collect();
        println!("All: {:?}", all);


        car = car.difference(&all).cloned().collect();
        car = car.difference(&bike).cloned().collect();
        println!("Car: {:?}", car);

        bike = bike.difference(&all).cloned().collect();
        bike = bike.difference(&car).cloned().collect();
        println!("Bike: {:?}", bike);
    }
}
