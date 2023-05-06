use argmin::prelude::ArgminOp;

use crate::prelude::*;
use argmin::prelude::*;

struct Problem<K, F> {
    random: std::cell::RefCell<Random>,
    distance: F,
    key: std::marker::PhantomData<K>,
}

impl<K: std::fmt::Debug + Copy + serde::de::DeserializeOwned + Serialize, F: Fn(K, K) -> f64>
    ArgminOp for Problem<K, F>
{
    type Param = Vec<K>;
    type Output = f64;
    type Hessian = ();
    type Jacobian = ();
    type Float = f64;

    fn apply(&self, param: &Vec<K>) -> Result<f64, Error> {
        Ok(param
            .windows(2)
            .map(|window| (self.distance)(window[0], window[1]))
            .sum())
    }

    fn modify(&self, param: &Vec<K>, _temp: f64) -> Result<Vec<K>, Error> {
        let mut result = param.clone();
        let index = self.random.borrow_mut().gen_range(0..result.len());
        let index2 = self.random.borrow_mut().gen_range(0..result.len());
        let removed = result.remove(index);
        result.insert(index2, removed);
        Ok(result)
    }
}
/*
#[allow(unused)]
pub fn optimize_order<K: std::fmt::Debug + Copy + Serialize + serde::de::DeserializeOwned>(
    keys: Vec<K>,
    distance: impl Fn(K, K) -> f64,
) -> Vec<K> {
    let random = rand::rngs::StdRng::seed_from_u64(1337);
    let solver = argmin::solver::simulatedannealing::SimulatedAnnealing::new(15.0, 0)
        .unwrap()
        .stall_best(1000);
    let operator = Problem {
        random: std::cell::RefCell::new(random),
        key: std::marker::PhantomData::default(),
        distance,
    };
    let result = Executor::new(operator, solver, keys)
        .max_iters(1_000_000)
        .run()
        .unwrap();

    result.state().best_param.clone()
}
*/
