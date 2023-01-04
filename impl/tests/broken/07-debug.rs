use std::ops::Deref;

// Check that the simplest use of the #[wye] attribute proc-macro compiles.
use wye::*;
use pretty_assertions::{assert_eq};

struct OnlyDebug(&'static str);

impl std::fmt::Debug for OnlyDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl Deref for OnlyDebug {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

struct OnlyDisplay(&'static str);

impl std::fmt::Display for OnlyDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl Deref for OnlyDisplay {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[wye(a: format!("{a:?}"), b: format!("{b}"))]
fn concat(a: OnlyDebug, b: OnlyDisplay) -> String { format!("{:?} {}", &a, &b) }

const EXPECTED_GRAPH: &str = r#"
digraph {
    0 [ label = "aa = a" ]
    1 [ label = "bb = b" ]
    2 [ label = "a = a" ]
    3 [ label = "b = b" ]
    4 [ label = "format! (\"{:?} {}\", & a, & b)" ]
    5 [ label = "a b" ]
    0 -> 2 [ label = "" ]
    1 -> 3 [ label = "" ]
    2 -> 4 [ label = "" ]
    3 -> 4 [ label = "" ]
    4 -> 5 [ label = "" ]
}
"#;

pub fn main() {
    // BUG: this test is insufficiently precise; it needs to validate that
    // the reference / slice expressions passed to concat get handled via 
    // push_var().
    let aa = OnlyDebug("a");
    let bb = OnlyDisplay("b");
    assert_eq!(wyre!{
        concat(aa, bb)
    }, "a b");
    eprintln!("{}", get_wye());
    assert_eq!(format!("{}", get_wye()).trim(), EXPECTED_GRAPH.trim());
}