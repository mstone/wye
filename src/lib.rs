//! # wye
//! 
//! wye is a crate for instrumenting Rust code to record causal traces for debugging.
//! 
//! Consider a simple program:
//! 
//! fn add(a: u64, b: u64) -> u64 {
//!     a + b
//! }
//! 
//! fn example() -> bool {
//!     let result = add(1, 2);
//!     result == 3
//! }
//! 
//! A causal trace for the execution of `add(...)` in example might be:
//! 
//! digraph {
//!     1 [ label="a = 1" ];
//!     2 [ label="b = 2" ];
//!     3 [ label="+" ];
//!     4 [ label="ret = 3" ];
//!     1 -> 3;
//!     2 -> 3;
//!     3 -> 4;
//! }
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