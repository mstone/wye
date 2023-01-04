// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;
use pretty_assertions::{assert_eq};

#[wye]
fn concat(a: String, b: String) -> String { format!("{} {}", a, b) }

const EXPECTED_GRAPH: &str = r#"digraph {
    0 [ label = "a = a" ]
    1 [ label = "b = b" ]
    2 [ label = "format! (\"{} {}\", a, b)" ]
    3 [ label = "a b" ]
    0 -> 2 [ label = "" ]
    1 -> 2 [ label = "" ]
    2 -> 3 [ label = "" ]
}
"#;

pub fn main() {
    assert_eq!(wyre!{
        concat("a".into(), "b".into())
    }, "a b");
    eprintln!("{}", get_wye());
    assert_eq!(&format!("{}", get_wye()), EXPECTED_GRAPH);
}