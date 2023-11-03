use procmacros::{Display, FromStr};
use serde::{Deserialize, Serialize};

use crate::models::Fissure;

pub trait Filter {
    fn apply_filter(&self, value: &Fissure) -> bool;
}

/// Whether to include, exclude or exclusively use the given value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, FromStr)]
pub enum ExclusivityFilter {
    /// Exclude those with the given value
    Exclude,
    /// Include those with and without the given value
    Include,
    /// Exclude all but these values
    Exclusive,
}
impl Filter for ExclusivityFilter {
    fn apply_filter(&self, value: &Fissure) -> bool {
        match self {
            ExclusivityFilter::Exclude => !value.is_storm,
            ExclusivityFilter::Include => true,
            ExclusivityFilter::Exclusive => value.is_storm,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, FromStr)]
pub enum Factions {
    Orokin,
    Grineer,
    Corpus,
    Infested,
    Narmer,
    Crossfire,
}
impl Filter for Factions {
    fn apply_filter(&self, value: &Fissure) -> bool {
        value.enemy == *self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, FromStr)]
pub enum Tier {
    Lith,
    Meso,
    Neo,
    Axi,
    Requiem,
}
impl Filter for Tier {
    fn apply_filter(&self, value: &Fissure) -> bool {
        value.tier == *self
    }
}
