use crate::traits::HgBasis;
use crate::structs::GeneroGraph;

use super::EdgeWeight;

/// A simple path structure for building paths in a graph. Currently
/// does not store a path reference so multiple graphs can be used
/// to extend paths, probably should fix. Also stores accumulated
/// weight along the path.
#[derive(Debug, Clone)]
pub struct HgPath<B: HgBasis> {
    sites: Vec<B>,
    weight: EdgeWeight,
}

impl<B: HgBasis> HgPath<B> {
    pub fn new(start: B) -> Self {
        HgPath { sites: vec![start] , weight:
        0.0}
    }

    /// How many basis elements the path has visited
    pub fn len(&self) -> usize {
        self.sites.len()
    }

    /// Returns the most recently visited basis if the path has non-zero length
    pub fn last_basis(&self) -> Option<B> {
        self.sites.last().map(|b| b.clone())
    }

    /// Returns the first visited basis if the path has non-zero length
    pub fn first_basis(&self) -> Option<B> {
        self.sites.first().map(|b| b.clone())
    }

    /// The accumulated weight of the path
    pub fn weight(&self) -> EdgeWeight {
        self.weight
    }

    /// Takes this path as an input and returns a list of paths that extend from
    /// the last site.
    pub fn extend(self, graph: &GeneroGraph<B>) -> Vec<HgPath<B>> {
        if self.sites.len() == 0 {
            return Vec::new();
        }
        let mut ret = Vec::new();
        let outputs = graph.map_basis(self.sites.last().expect("Checked for 0 length previously."));
        for (new_site, weight) in outputs.to_tuples() {
            let tmp = self.clone();
            tmp.sites.push(new_site);
            tmp.weight += weight;
            ret.push(tmp);
        }
        ret
    }
}

mod test {
    use uuid::Uuid;

    use crate::{algs::builders::erdos_renyi, structs::{path::HgPath, SparseBasis}, traits::HgBasis};

    #[test]
    fn test_simple_extension() {
        let hg = erdos_renyi::<Uuid>(10, vec![
            (0, 1, 1.),
            (0, 2, 1.),
        ]);
        println!("hg: {:#?}", hg);
        // let p = HgPath::new(SparseBasis::new_empty());
        // println!("path extensions:\n{:#?}", p.extend(&hg));
    }
}