use procmacros::{Display, FromStr};
use serde::{Deserialize, Serialize};

use crate::models::Fissure;


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
impl ExclusivityFilter {
    pub fn apply_filter(&self, value: bool) -> bool {
        match self {
            ExclusivityFilter::Exclude => !value,
            ExclusivityFilter::Include => true,
            ExclusivityFilter::Exclusive => value,
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
impl Factions {
    pub fn apply_filter(&self, value: &Fissure) -> bool {
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
impl Tier {
    pub fn apply_filter(&self, value: &Fissure) -> bool {
        value.tier == *self
    }
}
