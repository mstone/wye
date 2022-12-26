// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;
use pretty_assertions::{assert_eq};

#[wye]
fn add(a: u64, b: u64) -> u64 { a + b }

const EXPECTED_GRAPH: &str = r#"digraph {
    0 [ label = "a = 2" ]
    1 [ label = "b = 3" ]
    2 [ label = "+" ]
    3 [ label = "5" ]
    4 [ label = "five = 5" ]
    5 [ label = "a = 1" ]
    6 [ label = "b = 5" ]
    7 [ label = "+" ]
    8 [ label = "6" ]
    0 -> 2 [ label = "" ]
    1 -> 2 [ label = "" ]
    2 -> 3 [ label = "" ]
    3 -> 4 [ label = "" ]
    4 -> 6 [ label = "" ]
    5 -> 7 [ label = "" ]
    6 -> 7 [ label = "" ]
    7 -> 8 [ label = "" ]
}
"#;

pub fn main() {
    assert_eq!(wyre!{
        let five = add(2, 3);
        add(1, five)
    }, 6);
    eprintln!("{}", get_wye());
    assert_eq!(&format!("{}", get_wye()), EXPECTED_GRAPH);
}