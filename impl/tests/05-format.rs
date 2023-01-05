// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;
use pretty_assertions::{assert_eq};

#[wye]
fn concat(a: String, b: String) -> String { format!("{} {}", a, b) }

const EXPECTED_GRAPH: &str = r#"
digraph {
    0 [ label = "concat(Into::<String>::into(\"a\"), Into::<String>::into(\"b\")) = a b" ]
    1 [ label = "Into::<String>::into(\"a\") = a" ]
    2 [ label = "a" ]
    3 [ label = "Into::<String>::into(\"b\") = b" ]
    4 [ label = "b" ]
    5 [ label = "a = a" ]
    6 [ label = "b = b" ]
    7 [ label = "format!(\"{} {}\", a, b) = a b" ]
    2 -> 1 [ label = "" ]
    4 -> 3 [ label = "" ]
    1 -> 5 [ label = "" ]
    3 -> 6 [ label = "" ]
    5 -> 7 [ label = "" ]
    6 -> 7 [ label = "" ]
    7 -> 0 [ label = "" ]
}
"#;

pub fn main() {
    assert_eq!(wyre!{
        concat(Into::<String>::into("a"), Into::<String>::into("b"))
    }, "a b");
    eprintln!("{}", get_wye());
    assert_eq!(format!("{}", get_wye()).trim(), EXPECTED_GRAPH.trim());
}