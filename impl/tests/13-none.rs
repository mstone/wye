// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;
use pretty_assertions::{assert_eq};

#[wye]
fn none(a: u64, b: u64) -> Option<u64> { 
    let maybe_val = None;
    return maybe_val;
}

const EXPECTED_GRAPH: &str = r#"digraph {
}
"#;

pub fn main() {
    assert_eq!(wyre!{
        none(1, 2).unwrap_or(3)
    }, 3);
    eprintln!("{}", get_wye());
    assert_eq!(&format!("{}", get_wye()), EXPECTED_GRAPH);
}