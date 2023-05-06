mod amino_acids;
pub mod groups;
pub mod moves;
mod nn_join;
mod parameter;

pub use groups::optimize_groups;

pub use amino_acids::analyze_amino_acids;
pub use nn_join::nn_join;
pub use parameter::optimize_parameter;


use crate::prelude::*;
pub fn optimize(graph: &mut Graph) -> Vec<moves::MoveLog> {
	let made_moves = moves::optimize(graph);
	optimize_groups(graph);
	optimize_parameter(graph);
	graph.compact();

	made_moves
}