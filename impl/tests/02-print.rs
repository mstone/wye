// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;

#[wye]
fn add(a: u64, b: u64) -> u64 { a + b }

const EXPECTED_GRAPH: &str = r#"digraph {
    0 [ label = "a = 1" ]
    1 [ label = "b = 2" ]
    2 [ label = "+ = 3" ]
    0 -> 2 [ label = "" ]
    1 -> 2 [ label = "" ]
}
"#;

pub fn main() {
    assert_eq!(add(1, 2), 3);
    eprintln!("{}", get_wye());
    assert_eq!(&format!("{}", get_wye()), EXPECTED_GRAPH);
}