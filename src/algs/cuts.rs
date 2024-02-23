use crate::{structs::EdgeWeight, HgBasis};

pub fn cut<B: HgBasis>(_inputs: &B, _outputs: &B) -> EdgeWeight {
    1.
}
