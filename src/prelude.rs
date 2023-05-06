pub use color_eyre::eyre::Result;
pub use color_eyre::eyre::{bail, eyre};

#[cfg(test)]
pub use quickcheck::quickcheck;

pub use noisy_float::prelude::*;

pub use itertools::Itertools;
pub use log::{debug, info, trace};
pub use rayon::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use std::convert::TryFrom;

pub use either::Either;
pub use fnv::FnvHashMap;
pub use rand::Rng;
pub use rayon::iter::ParallelIterator;
pub use sorted_vec::SortedSet;
pub type Random = rand::rngs::StdRng;

pub use crate::alignment::{read_alignment, Alignment, PositionData, PositionIndex, SequenceId};
pub use crate::amino_acids::{AminoAcid, AminoAcidMap};
pub use crate::fixed::{FixedIndex, FixedVec};
pub use crate::graph::Graph;
pub use crate::graph::NodeId;
pub use crate::graph::NodeKind;
pub use crate::log::{FixedLog, Log};
pub use crate::slab::Slab;
pub use crate::slab::SlabMap;
pub use crate::slab::SlabSet;
pub use crate::trace::NullTracer;
pub use crate::trace::Tracer;
pub use crossterm::{cursor, execute, queue, style, terminal};
pub use std::io::stdout;
pub use triomphe::Arc;

pub type AminoAcidMatrix<T> =
    nalgebra::Matrix<T, nalgebra::U20, nalgebra::U20, nalgebra::ArrayStorage<T, 20, 20>>;
