use std::fmt::{Display};

// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;
use pretty_assertions::{assert_eq};

struct Datum<V, E>(V, E) where V: Display, E: Display;

impl<V, E> Display for Datum<V, E> where V: Display, E: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)?;
        write!(f, " ")?;
        self.1.fmt(f)
    }
}

#[wye]
fn concat<V, E>(a: V, b: E) -> String where V: Display, E: Display { format!("{} {}", a, b) }

const EXPECTED_GRAPH: &str = r#"
digraph {
    0 [ label = "aa = a a" ]
    1 [ label = "bb = b b" ]
    2 [ label = "a = a a" ]
    3 [ label = "b = b b" ]
    4 [ label = "format! (\"{} {}\", a, b)" ]
    5 [ label = "a a b b" ]
    0 -> 2 [ label = "" ]
    1 -> 3 [ label = "" ]
    2 -> 4 [ label = "" ]
    3 -> 4 [ label = "" ]
    4 -> 5 [ label = "" ]
}
"#;

pub fn main() {
    let aa = Datum("a", "a");
    let bb = Datum("b", "b");
    assert_eq!(
        wyre!{
            concat(aa, bb)
        }
        , "a a b b");
    eprintln!("{}", get_wye());
    assert_eq!(format!("{}", get_wye()).trim(), EXPECTED_GRAPH.trim());
}