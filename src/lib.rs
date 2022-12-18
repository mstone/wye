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

pub mod collections {
    pub struct Vec {}

    pub struct HashMap {}

    pub struct HashSet {}

    pub struct BTreeMap {}

    pub struct BTreeSet {}

}

