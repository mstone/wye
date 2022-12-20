//! # wye
//! 
//! wye is a crate for instrumenting Rust code to record causal traces for debugging.
//! 
//! Consider a simple program:
//! 
//! #[wye]
//! fn add(a: u64, b: u64) -> u64 {
//!     a + b
//! }
//! 
//! fn example() -> bool {
//!     add(1, add(2, 3)) == 6
//! }
//! 
//! A causal trace for the execution of `add(...)` in example might be presented by:
//! 
//! digraph {
//!     1 [ label="a = 2" ];
//!     2 [ label="b = 3" ];
//!     3 [ label="+" ];
//!     4 [ label="5" ];
//!     5 [ label="a = 1"];
//!     6 [ label="b = 5"];
//!     7 [ label="+"];
//!     8 [ label="6"];
//!     1 -> 3;
//!     2 -> 3;
//!     3 -> 4;
//!     4 -> 6;
//!     5 -> 7;
//!     6 -> 7;
//!     7 -> 8;
//! }
//! 
//! To produce this, we need to transform add to
//! 
//! fn add(a: u64, b: u64) -> u64 {
//!     let ~w = wye_ref(WYE);
//!     let a = wye!{a}; // let a = wye::Value(a);
//!     let b = wye!{b}; // let b = wye::Value(b);
//!     let r1 = a + b;
//!     let l1 = wye_place!{};
//!     l1.set(r1);
//!     l1.into();
//! }
//! 
//! which when executed, needs to produce
//! 
//! let a = w.node("a", arg!{a});
//! let b = w.node("b", arg!{b});
//! let r1 = w.node("", "+");
//! let t1 = 2 + 3;
//! let l1 = w.node("", r1)
//! w.edge(a, o1);
//! w.edge(b, o1);
//! w.edge(r1, l1);
//! ret!{l1};
//! 
//! where arg! has the effect of determining whether or not the value in a is the result of a ret! or a set!, and where
//! ret! has the effect of registering l1 as a non-root (such that subsequent calls to arg! or get! will connect the
//! resulting graph.
//! 
//! (This seems like something that can maaaaybe be done either by wye! doing something fancy with function calls, or by
//! registering the addresses of places used by ret!/set!? Or the source locations?)
//! 
//! # See Also
//! 
//! * [rr](https://rr-project.org)
//! * [PANDA](https://github.com/panda-re/panda)
//! * [pernosco](https://pernos.co)

pub mod log {
    use petgraph::graph::Graph;
    pub struct Logger {
        graph: Graph<String, String>,
    }

    impl Logger {
        fn new() -> Self {
            Self {
                graph: Graph::new(),
            }
        }
    }
}

pub mod expr {
    pub struct Place {

    }

    pub struct Rvalue {

    }
}

pub mod iter {

}

pub mod collections {
    pub struct Vec {}

    pub struct HashMap {}

    pub struct HashSet {}

    pub struct BTreeMap {}

    pub struct BTreeSet {}

}