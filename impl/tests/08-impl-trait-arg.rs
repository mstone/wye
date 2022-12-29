use std::fmt::Display;

// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;
use pretty_assertions::{assert_eq};

#[wye]
fn concat(a: impl AsRef<str> + Display, b: impl AsRef<str> + Display) -> String { format!("{} {}", &a, b.as_ref()) }

const EXPECTED_GRAPH: &str = r#"
digraph {
    0 [ label = "aa = a" ]
    1 [ label = "bb = b" ]
    2 [ label = "a = a" ]
    3 [ label = "b = b" ]
    4 [ label = "format! (\"{} {}\", & a, b.as_ref())" ]
    5 [ label = "a b" ]
    0 -> 2 [ label = "" ]
    1 -> 3 [ label = "" ]
    2 -> 4 [ label = "" ]
    3 -> 4 [ label = "" ]
    4 -> 5 [ label = "" ]
}
"#;

pub fn main() {
    let aa = "a";
    let bb = "b";
    assert_eq!(wyre!{
        concat(aa, bb)
    }, "a b");
    eprintln!("{}", get_wye());
    assert_eq!(format!("{}", get_wye()).trim(), EXPECTED_GRAPH.trim());
}