use std::ops::Add;

use crate::structs::GeneroGraph;
use crate::traits::HgBasis;

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
        HgPath {
            sites: vec![start],
            weight: 0.0,
        }
    }

    /// How many edges the path has traversed. A path with a single basis
    /// element is length 0, a path with two basis elements is length 1, etc.
    /// Note: path must be non-empty and start from somewhere. vec must never be empty.
    pub fn len(&self) -> usize {
        self.sites.len() - 1
    }

    /// Returns the most recently visited basis if the path has non-zero length
    pub fn last_basis(&self) -> B {
        self.sites
            .last()
            .map(|b| b.clone())
            .expect("path should be non-empty.")
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
            let mut tmp = self.clone();
            tmp.sites.push(new_site);
            tmp.weight += weight;
            ret.push(tmp);
        }
        ret
    }
}

impl<B: HgBasis> Add<(B, EdgeWeight)> for HgPath<B> {
    type Output = HgPath<B>;

    fn add(self, rhs: (B, EdgeWeight)) -> Self::Output {
        let mut new_sites = self.sites;
        new_sites.push(rhs.0);
        HgPath {
            sites: new_sites,
            weight: self.weight + rhs.1,
        }
    }
}

mod test {
    use std::collections::HashSet;

    use crate::{structs::HgPath, HGraph, SparseNodeSet};

    #[test]
    fn test_simple_extension() {
        let mut hg = HGraph::new();
        let mut nodes = hg.add_nodes(10);
        nodes.sort();
        let p = HgPath::new(SparseNodeSet::from(HashSet::from([nodes[0]])));
        println!("nodes: {:#?}", nodes);
    }
}
