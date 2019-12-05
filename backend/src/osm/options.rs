use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::osm::highway::Highway::{Cycleway, Footway, LivingStreet, Motorway, MotorwayLink, Path,
                                   Pedestrian, Primary, PrimaryLink, Residential, Road, Secondary,
                                   SecondaryLink, Service, Steps,
                                   Tertiary, TertiaryLink, Track, Trunk, TrunkLink, Unclassified};
use crate::osm::highway::Highway;
use crate::osm::options::Transport::{All, Bike, BikeWalk, Car, CarBike, Walk};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Transport {
    Car,
    Bike,
    Walk,

    All,
    CarBike,
    BikeWalk,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Charging {
    Car,
    Bike,
    CarBike,
    None,
}

impl Charging {
    pub fn from(transport: Transport) -> Self {
        match transport {
            Bike => Charging::Bike,
            Car => Charging::Car,
            CarBike => Charging::CarBike,
            _ => Charging::None
        }
    }

    pub fn contains(self, other: Self) -> bool {
        self == Charging::CarBike || self == other
            || self == Charging::CarBike && (other == Charging::Car || other == Charging::Bike)
    }
}

impl Transport {
    pub fn from(highway: Highway) -> Self {
        match highway {
            Residential | Tertiary | Unclassified | Service | LivingStreet | TertiaryLink => All,
            Secondary | SecondaryLink | Primary | PrimaryLink => CarBike,
            Track | Road => BikeWalk,
            Motorway | MotorwayLink | Trunk | TrunkLink => Car,
            Cycleway => Bike,
            Pedestrian | Footway | Path | Steps => Walk,
        }
    }

    pub fn contains(self, other: Self) -> bool {
        self == All || self == other ||
            (self == CarBike && (other == Car || other == Bike)) ||
            (self == BikeWalk && (other == Bike || other == Walk))
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

    use crate::osm::highway::Highway::{Cycleway, Footway, LivingStreet, Motorway, MotorwayLink, Path,
                                       Pedestrian, Primary, PrimaryLink, Residential, Road, Secondary,
                                       SecondaryLink, Service, Steps,
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
        let mut walk: HashSet<Highway> = [
            Tertiary,
            Unclassified,
            Residential,
            TertiaryLink,
            LivingStreet,
            Service,
            Pedestrian,
            Track,
            Road,
            Footway,
            Steps,
            Path,
        ].iter().cloned().collect();

        let mut all: HashSet<Highway> = car.intersection(&bike).cloned().collect();
        all = all.intersection(&walk).cloned().collect();
        println!("All: {:?}", all);

        let mut car_bike: HashSet<Highway> = car.intersection(&bike).cloned().collect();
        car_bike = car_bike.difference(&all).cloned().collect();
        println!("CarBike: {:?}", car_bike);

        let mut bike_walk: HashSet<Highway> = bike.intersection(&walk).cloned().collect();
        bike_walk = bike_walk.difference(&all).cloned().collect();
        bike_walk = bike_walk.difference(&car_bike).cloned().collect();
        println!("BikeWalk: {:?}", bike_walk);

        car = car.difference(&all).cloned().collect();
        car = car.difference(&car_bike).cloned().collect();
        car = car.difference(&bike_walk).cloned().collect();
        car = car.difference(&bike).cloned().collect();
        car = car.difference(&walk).cloned().collect();
        println!("Car: {:?}", car);

        bike = bike.difference(&all).cloned().collect();
        bike = bike.difference(&car_bike).cloned().collect();
        bike = bike.difference(&bike_walk).cloned().collect();
        bike = bike.difference(&car).cloned().collect();
        bike = bike.difference(&walk).cloned().collect();
        println!("Bike: {:?}", bike);

        walk = walk.difference(&all).cloned().collect();
        walk = walk.difference(&car_bike).cloned().collect();
        walk = walk.difference(&bike_walk).cloned().collect();
        walk = walk.difference(&car).cloned().collect();
        walk = walk.difference(&bike).cloned().collect();
        println!("Walk: {:?}", walk);
    }
}
