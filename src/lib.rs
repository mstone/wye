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

use std::{fmt::Display, sync::Once, collections::HashMap};

pub use wye_impl::{wye, wyre};

pub struct Wye {
    graph: petgraph::graph::Graph<String, String>,
    nodes: HashMap<(u64, u64), petgraph::graph::NodeIndex>,
    frames: Vec<Vec<Option<(u64, u64)>>>,
    last_node: Option<(u64, u64)>,
}

impl Wye {
    fn new() -> Self {
        Wye {
            graph: petgraph::graph::Graph::new(),
            nodes: HashMap::new(),
            frames: vec![vec![]],
            last_node: None,
        }
    }

    pub fn node<V: Display>(&mut self, frame: u64, slot: u64, var: Option<impl Display>, val: V) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.nodes.entry((frame, slot)) {
            let weight = var.map(|var| format!("{var} = {val}")).unwrap_or_else(|| format!("{val}"));
            let node = self.graph.add_node(weight);
            e.insert(node);
            self.last_node = Some((frame, slot));
        }
    }

    pub fn edge(&mut self, from_frame: u64, from_slot: u64, to_frame: u64, to_slot: u64) {
        let from = self.nodes[&(from_frame, from_slot)];
        let to = self.nodes[&(to_frame, to_slot)];
        self.graph.add_edge(from, to, "".into());
    }

    pub fn push_frame(&mut self) {
        self.frames.push(vec![]);
    }

    pub fn pop_frame(&mut self) {
        self.frames.pop();
    }

    pub fn push_lit(&mut self) {
        self.frames.last_mut().unwrap().push(None);
    }

    pub fn push_var(&mut self, addr: (u64, u64)) {
        self.frames.last_mut().unwrap().push(Some(addr));
    }

    pub fn frame(&self) -> (u64, Vec<Option<(u64, u64)>>) {
        let fno = self.frames.len() as u64 - 1;
        (fno, self.frames.last().unwrap().clone())
    }

    pub fn last_node(&self) -> (u64, u64) {
        self.last_node.unwrap()
    }
}

impl Display for Wye {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dot = petgraph::dot::Dot::new(&self.graph);
        dot.fmt(f)
    }
}

static mut WYE: Option<Wye> = None;
static mut INIT: std::sync::Once = Once::new();

pub fn get_wye<'a>() -> &'a mut Wye {
    unsafe {
        INIT.call_once(|| {
            WYE = Some(Wye::new());
        });
        WYE.as_mut().unwrap()
    }
}