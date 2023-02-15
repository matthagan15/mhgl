use crate::traits::HgNode;

pub struct PGraph<N: HgNode> {
    data: Vec<N>,
}