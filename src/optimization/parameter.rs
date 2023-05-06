use argmin::prelude::ArgminOp;

use crate::{amino_acids::AminoAcidModel, prelude::*};
use argmin::prelude::*;

struct Problem<'a> {
    amino_acid_model: &'a AminoAcidModel,
    counts: AminoAcidMatrix<i32>,
}

impl<'a> ArgminOp for Problem<'a> {
    type Param = Vec<f64>;
    type Output = f64;
    type Hessian = ();
    type Jacobian = ();
    type Float = f64;

    fn apply(&self, param: &Vec<f64>) -> Result<f64, Error> {
        let param: R64 = r64(param[0]);
        if param < 0.0 {
            return Ok(f64::INFINITY);
        }
        let result: N64 = self
            .amino_acid_model
            .parameterize(param)
            .likelihood(&self.counts)
            .log2();

        Ok(-result.raw())
    }
}

pub fn optimize_parameter(graph: &mut Graph) {
    let solver = argmin::solver::neldermead::NelderMead::new().with_initial_params(vec![
        vec![graph.parameter().raw()],
        vec![graph.parameter().raw() - 0.1],
        vec![graph.parameter().raw() + 0.1],
    ]).sd_tolerance(0.1);
    graph.ensure_clean();
    let operator = Problem {
        counts: graph.stats.transitions,
        amino_acid_model: graph.amino_acid_model(),
    };

    let baseline = graph.parameterized_model().likelihood(&operator.counts);

    let result = Executor::new(operator, solver, vec![graph.parameter().raw()])
		.max_iters(1_000_000)
        .run()
        .unwrap();

    let state = result.state();

    if Log::pow2(n64(-state.best_cost)) > baseline {
        graph.set_parameter(r64(state.best_param[0]));
    }
}
