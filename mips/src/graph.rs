use std::collections::{BTreeMap, BTreeSet};
use std::{ fmt, fmt::Debug};

use itertools::join;

#[derive(Clone)]
pub struct Node {
    index: usize,
    color: Option<usize>,
    edges: BTreeSet<usize>,
}

impl Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let index = self.index;
        let join_edges = || join(self.edges.iter(), ",");
        match (self.color, self.edges.is_empty()) {
            (Some(color), false) => write!(f, "Node({},{}) -> [{}]", index, color, join_edges()),
            (None, false) => write!(f, "Node({},None) -> [{}]", index, join_edges()),
            (Some(color), true) => write!(f, "Node({},{})", index, color),
            (None, true) => write!(f, "Node({},None)", index),
        }
    }
}

impl Node {
    pub fn new(index: usize) -> Self {
        let color = None;
        let edges = BTreeSet::new();
        Node {
            index,
            color,
            edges,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn color(&self) -> Option<usize> {
        self.color.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Graph {
    nodes: BTreeMap<usize, Node>,
}

impl Default for Graph {
    fn default() -> Self {
        let nodes = BTreeMap::new();
        Self { nodes }
    }
}

impl Graph {
    pub fn add_node(&mut self, i: usize) {
        if !self.nodes.contains_key(&i) {
            let node = Node::new(i);
            self.nodes.insert(i, node);
        }
    }

    fn add_edge_directed(&mut self, from: usize, to: usize) {
        let from_node = self.nodes.get_mut(&from).unwrap();
        from_node.edges.insert(to);
    }

    pub fn add_edge(&mut self, i: usize, j: usize) {
        self.add_edge_directed(i, j);
        self.add_edge_directed(j, i);
    }

    pub fn with_edges<T>(mut self, edges: T) -> Self
    where
        T: IntoIterator<Item = (usize, usize)>,
    {
        for (a, b) in edges.into_iter() {
            self.nodes.entry(a).or_insert(Node::new(a));
            if a != b {
                self.nodes.entry(b).or_insert(Node::new(b));
                self.add_edge(a, b);
            }
        }
        self
    }

    pub fn from_edges<T>(edges: T) -> Self
    where
        T: IntoIterator<Item = (usize, usize)>,
    {
        Self::default().with_edges(edges.into_iter())
    }

    pub fn into_nodes(self) -> impl Iterator<Item = Node> {
        self.nodes.into_values()
    }

    fn adjacent_colors(&self, from: &usize) -> BTreeSet<usize> {
        self.nodes[from]
            .edges
            .iter()
            .filter_map(|to| self.nodes[to].color.clone())
            .collect()
    }

    fn try_n_color(&self, n: usize) -> Result<Self, ()> {
        let colors = (0..n).collect::<BTreeSet<_>>();
        let mut graph = self.clone();
        for i in self.nodes.keys() {
            let adjacent_colors = graph.adjacent_colors(&i);
            let opt = colors.difference(&adjacent_colors).next();
            if let Some(color) = opt {
                let node = graph.nodes.get_mut(&i).unwrap();
                node.color = Some(*color);
            } else {
                return Err(());
            }
        }
        Ok(graph)
    }

    pub fn color(&self) -> Self {
        for n in 1.. {
            if let Ok(colored) = self.try_n_color(n) {
                return colored;
            }
        }
        unreachable!();
    }
}


