use std::fmt::Display;

// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;
use pretty_assertions::{assert_eq};

struct Multiple {
    a: u64,
    b: u64,
}

impl Display for Multiple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.a, self.b)
    }
}

#[wye]
fn add(m: Multiple) -> u64 { 
    let Multiple{a, b, ..} = m;
    a + b 
}

const EXPECTED_GRAPH: &str = r#"digraph {
    0 [ label = "a = 2" ]
    1 [ label = "b = 3" ]
    2 [ label = "Multiple{a: 2, b: 3}" ]
    3 [ label = "m = 2 3" ]
    4 [ label = "+" ]
    5 [ label = "5" ]
    0 -> 2 [ label = "" ]
    1 -> 2 [ label = "" ]
    2 -> 3 [ label = "" ]
    3 -> 3 [ label = "" ]
    3 -> 0 [ label = "" ]
    3 -> 1 [ label = "" ]
    0 -> 4 [ label = "" ]
    1 -> 4 [ label = "" ]
    4 -> 5 [ label = "" ]
}
"#;

pub fn main() {
    assert_eq!(
        wyre!{
            let m = Multiple{a: 2, b: 3};
            add(m)
        }
    , 5);
    eprintln!("{}", get_wye());
    assert_eq!(format!("{}", get_wye()).trim(), EXPECTED_GRAPH.trim());
}