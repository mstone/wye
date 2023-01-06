//! # Overview
//! 
//! wye is a crate for instrumenting Rust code to record causal traces for debugging.
//! 
//! # Example
//! 
//! Consider a simple program:
//! 
//! ```rust
//! #[wye]
//! fn add(a: u64, b: u64) -> u64 {
//!     a + b
//! }
//! 
//! fn example() -> bool {
//!     wyre!{add(1, add(2, 3))} == 6
//! }
//! ```
//! 
//! A causal trace for the execution of `add(...)` in example might be presented by:
//! 
//! ```dot
//! digraph {
//! 0 [ label = "a = 2" ]
//! 1 [ label = "b = 3" ]
//! 2 [ label = "+" ]
//! 3 [ label = "5" ]
//! 4 [ label = "a = 1" ]
//! 5 [ label = "b = 5" ]
//! 6 [ label = "+" ]
//! 7 [ label = "6" ]
//! 0 -> 2 [ label = "" ]
//! 1 -> 2 [ label = "" ]
//! 2 -> 3 [ label = "" ]
//! 3 -> 5 [ label = "" ]
//! 4 -> 6 [ label = "" ]
//! 5 -> 6 [ label = "" ]
//! 6 -> 7 [ label = "" ]
//! }
//! ```
//! 
//! # Guide-level explanation
//! 
//! [wye] transforms the functions it is used to annotate to record dataflow 
//! from arguments to returned results.
//! 
//! [wyre] transforms the expressions -- typically call-sites -- that it spans
//! to record data-flow from variables to arguments of the functions being called.
//! 
//! # Options
//! 
//! ## Custom Formatting
//! 
//! [wye] and [wyre] default to printing values via [std::fmt::Display] 
//! (technically, [ToString]) but this default choice can be overriden on a
//! per-argument basis by giving [wye] or [wyre] an expression for each 
//! argument to be specially printed, like:
//! 
//! ```rust
//! #[wye(a: format!("{a:?}"), b: format!("{b:?}"))]
//! fn concat(a: impl Debug + Display, b: impl Debug + Display) {
//!     format!("{a} {b}")
//! }
//! ```
//! 
//! or
//! 
//! ```rust
//! fn add(a: u64, b: u64) -> u64 {
//!     wyre!{ (a: format!("{a:?}"), b: format!("{b:?}"))
//!         a + b
//!     }
//! }
//! ```
//! 
//! # See Also
//! 
//! * [rr](https://rr-project.org)
//! * [PANDA](https://github.com/panda-re/panda)
//! * [pernosco](https://pernos.co)

use std::{fmt::{Display}, sync::Once, collections::HashMap};

pub use wye_impl::{wye, wyre};

pub struct Logger {
    graph: petgraph::graph::Graph<String, String>,
    nodes: HashMap<(u64, u64), petgraph::graph::NodeIndex>,
    frames: Vec<Vec<Option<(u64, u64)>>>,
    last_node: Option<(u64, u64)>,
    epoch: u64,
}

impl Logger {
    fn new() -> Self {
        Self {
            graph: petgraph::graph::Graph::new(),
            nodes: HashMap::new(),
            frames: vec![vec![]],
            last_node: None,
            epoch: 0,
        }
    }

    pub fn node(&mut self, frame: u64, slot: u64, var: Option<String>, val: String) {
        self.declare_node(frame, slot);
        self.define_node(frame, slot, var, val);
    }

    pub fn declare_node(&mut self, frame: u64, slot: u64) {
        match self.nodes.entry((frame, slot)) {
            std::collections::hash_map::Entry::Occupied(_) => {
                panic!("already declared node: {frame}, {slot}");
            },
            std::collections::hash_map::Entry::Vacant(ve) => {
                let node = self.graph.add_node(Default::default());
                ve.insert(node);
            },
        }
    }

    pub fn define_node(&mut self, frame: u64, slot: u64, var: Option<String>, val: String) {
        match self.nodes.entry((frame, slot)) {
            std::collections::hash_map::Entry::Occupied(oe) => {
                let weight = var.as_ref()
                    .map(|var| format!("{var} = {val}"))
                    .unwrap_or_else(|| val.clone());
                let node = oe.get();
                let node_weight = self.graph.node_weight_mut(*node)
                    .expect(&format!("missing node: {frame}, {slot} for update: {var:?} = {val}"));
                *node_weight = weight;
                self.last_node = Some((frame, slot));
            },
            std::collections::hash_map::Entry::Vacant(ve) => {
                panic!("undefined node: {frame}, {slot}");
            },
        }
    }

    pub fn edge(&mut self, from_frame: u64, from_slot: u64, to_frame: u64, to_slot: u64) {
        let from = self.nodes.get(&(from_frame, from_slot)).copied().unwrap_or_else(|| panic!("no entry found for from key: {from_frame}, {from_slot}"));
        let to = self.nodes.get(&(to_frame, to_slot)).copied().unwrap_or_else(|| panic!("no entry found for to key: {to_frame}, {to_slot}"));
        self.graph.add_edge(from, to, "".into());
    }

    pub fn push_frame(&mut self) {
        self.frames.push(vec![]);
    }

    pub fn pop_frame(&mut self) {
        self.frames.pop();
        self.epoch += 1;
    }

    pub fn push_lit(&mut self) {
        self.frames.last_mut().unwrap().push(None);
    }

    pub fn push_var(&mut self, addr: (u64, u64)) {
        self.frames.last_mut().unwrap().push(Some(addr));
    }

    pub fn frame(&self) -> (u64, Vec<Option<(u64, u64)>>) {
        (self.epoch, self.frames.last().unwrap().clone())
    }

    pub fn last_node(&self) -> (u64, u64) {
        self.last_node.unwrap()
    }

    pub fn set_last_node(&mut self, addr: (u64, u64)) {
        self.last_node = Some(addr);
    }
}

impl Display for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dot = petgraph::dot::Dot::new(&self.graph);
        dot.fmt(f)
    }
}

static mut WYE: Option<Logger> = None;
static mut INIT: std::sync::Once = Once::new();

pub fn get_wye<'a>() -> &'a mut Logger {
    unsafe {
        INIT.call_once(|| {
            WYE = Some(Logger::new());
        });
        WYE.as_mut().unwrap()
    }
}