use std::collections::HashMap;

use crate::structs::{EdgeWeight, GeneroVector};
use crate::traits::*;
/// Returns a monte carlo estimator to the average cardinality of the provided hypergraph. Essentially samples random basis vectors, applies the graph, and then takes the average of the cardinality random variable over these samples. allows you to specify the number of steps taken in the hypergraph.
fn average_cardinality<B, H>(
    hg: &H,
    num_samples: usize,
    num_steps: usize,
) -> HashMap<usize, EdgeWeight>
where
    B: HgBasis,
    H: HyperGraph<Basis = B>,
{
    let mut avg_card: HashMap<usize, EdgeWeight> = HashMap::new();
    for _ in 0..num_samples {
        // TODO: Need to change this from new vector to a random vector.
        let mut state_vec: GeneroVector<B> = GeneroVector::new();
        for _ in 0..num_steps {
            state_vec = hg.map_vector(&state_vec);
        }
        let card = state_vec.cardinality();
        for (k, p) in card {
            let old_p = avg_card.entry(k).or_insert(0.0);
            *old_p = *old_p + (p / (num_samples as f64));
        }
    }
    avg_card
}

/// Takes in a cardinality map and returns the expected value. Does not check if map is a probability vec.
fn expected_cardinality<N: HgNode>(card: &HashMap<usize, EdgeWeight>) -> EdgeWeight {
    let mut tot = 0.0;
    for (k, p) in card.iter() {
        tot += (*k as f64) * p;
    }
    tot
}
