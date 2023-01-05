// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;
use pretty_assertions::{assert_eq};

#[wye]
fn add(a: u64, b: u64) -> u64 { a + b }

const EXPECTED_GRAPH: &str = r#"
digraph {
    0 [ label = "add(1, add(2, 3)) = 6" ]
    1 [ label = "1" ]
    2 [ label = "add(2, 3) = 5" ]
    3 [ label = "2" ]
    4 [ label = "3" ]
    5 [ label = "a = 2" ]
    6 [ label = "b = 3" ]
    7 [ label = "+ = 5" ]
    8 [ label = "a = 1" ]
    9 [ label = "b = 5" ]
    10 [ label = "+ = 6" ]
    3 -> 5 [ label = "" ]
    4 -> 6 [ label = "" ]
    5 -> 7 [ label = "" ]
    6 -> 7 [ label = "" ]
    7 -> 2 [ label = "" ]
    1 -> 8 [ label = "" ]
    2 -> 9 [ label = "" ]
    8 -> 10 [ label = "" ]
    9 -> 10 [ label = "" ]
    10 -> 0 [ label = "" ]
}

"#;

pub fn main() {
    assert_eq!(wyre!{add(1, add(2, 3))}, 6);
    eprintln!("{}", get_wye());
    assert_eq!(format!("{}", get_wye()).trim(), EXPECTED_GRAPH.trim());
}