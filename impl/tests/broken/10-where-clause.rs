use std::fmt::{Display};

// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;
use pretty_assertions::{assert_eq};

struct Datum<V>(V) where V: Display;

impl<V> Display for Datum<V> where V: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[wye]
fn concat<V>(a: V, b: V) -> String where V: Display { format!("{} {}", a, b) }

const EXPECTED_GRAPH: &str = r#"
digraph {
    0 [ label = "aa = a" ]
    1 [ label = "bb = b" ]
    2 [ label = "a = a" ]
    3 [ label = "b = b" ]
    4 [ label = "format! (\"{} {}\", a, b)" ]
    5 [ label = "a b" ]
    0 -> 2 [ label = "" ]
    1 -> 3 [ label = "" ]
    2 -> 4 [ label = "" ]
    3 -> 4 [ label = "" ]
    4 -> 5 [ label = "" ]
}
"#;

pub fn main() {
    let aa = Datum("a");
    let bb = Datum("b");
    assert_eq!(
        wyre!{
            concat(aa, bb)
        }
        , "a b");
    eprintln!("{}", get_wye());
    assert_eq!(format!("{}", get_wye()).trim(), EXPECTED_GRAPH.trim());
}