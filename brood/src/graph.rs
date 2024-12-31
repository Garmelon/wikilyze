use std::ops::{Add, AddAssign, Range, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeIdx(pub u32);

impl NodeIdx {
    pub const NONE: Self = Self(u32::MAX);

    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(value as u32)
    }

    #[inline]
    pub const fn usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for NodeIdx {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<usize> for NodeIdx {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl Add for NodeIdx {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for NodeIdx {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for NodeIdx {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for NodeIdx {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Add<u32> for NodeIdx {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u32> for NodeIdx {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

impl Sub<u32> for NodeIdx {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<u32> for NodeIdx {
    fn sub_assign(&mut self, rhs: u32) {
        self.0 -= rhs;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeIdx(pub u32);

impl EdgeIdx {
    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(value as u32)
    }

    #[inline]
    pub const fn usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for EdgeIdx {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<usize> for EdgeIdx {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl Add for EdgeIdx {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for EdgeIdx {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for EdgeIdx {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for EdgeIdx {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Add<u32> for EdgeIdx {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u32> for EdgeIdx {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

impl Sub<u32> for EdgeIdx {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<u32> for EdgeIdx {
    fn sub_assign(&mut self, rhs: u32) {
        self.0 -= rhs;
    }
}

#[derive(Default)]
pub struct Graph {
    /// A node points to the first of its edges.
    ///
    /// A special case is that if the subsequent node points to the same edge,
    /// the current node has no edges.
    pub nodes: Vec<EdgeIdx>,

    /// An edge points to a target node.
    ///
    /// The source node is defined implicitly by the graph data structure.
    pub edges: Vec<NodeIdx>,
}

impl Graph {
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(nodes),
            edges: Vec::with_capacity(edges),
        }
    }

    pub fn add_node(&mut self) {
        self.nodes.push(EdgeIdx::new(self.edges.len()));
    }

    pub fn add_edge(&mut self, target: NodeIdx) {
        self.edges.push(target);
    }

    pub fn check_consistency(&self) {
        if self.nodes.is_empty() {
            assert!(self.edges.is_empty(), "edges must belong to existing nodes");
            return;
        }

        assert!(self.nodes.len() < u32::MAX as usize, "too many nodes");
        assert!(self.edges.len() < u32::MAX as usize, "too many edges");

        assert_eq!(
            *self.nodes.first().unwrap(),
            EdgeIdx(0),
            "first node pointer must be 0"
        );

        for (ni, node) in self.nodes.iter().cloned().enumerate() {
            assert!(
                node.usize() <= self.edges.len(),
                "node pointers must be in range"
            );

            if let Some(succ) = self.nodes.get(ni + 1) {
                assert!(node <= *succ, "node pointers must be well-ordered");
            }
        }

        for edge in &self.edges {
            assert!(
                edge.usize() < self.nodes.len(),
                "edge pointers must be in range"
            );
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = NodeIdx> + '_ {
        (0..self.nodes.len()).map(NodeIdx::new)
    }

    pub fn edges(&self) -> impl Iterator<Item = (NodeIdx, NodeIdx)> + '_ {
        Edges::new(self)
    }

    pub fn edges_for(&self, node: NodeIdx) -> impl Iterator<Item = (EdgeIdx, NodeIdx)> + '_ {
        self.edge_range(node)
            .map(|i| (EdgeIdx::new(i), self.edges[i]))
    }

    pub fn edge_start(&self, node: NodeIdx) -> EdgeIdx {
        self.nodes
            .get(node.usize())
            .copied()
            .unwrap_or_else(|| self.edges.len().into())
    }

    pub fn edge_range(&self, node: NodeIdx) -> Range<usize> {
        let start = self.nodes[node.usize()];
        let end = self.edge_start(node + 1);
        start.usize()..end.usize()
    }

    pub fn edge_slice(&self, node: NodeIdx) -> &[NodeIdx] {
        &self.edges[self.edge_range(node)]
    }
}

struct Edges<'a> {
    graph: &'a Graph,
    ni: NodeIdx,
    ei: EdgeIdx,
}

impl<'a> Edges<'a> {
    fn new(graph: &'a Graph) -> Self {
        Self {
            graph,
            ni: NodeIdx(0),
            ei: EdgeIdx(0),
        }
    }
}

impl Iterator for Edges<'_> {
    type Item = (NodeIdx, NodeIdx);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ei.usize() >= self.graph.edges.len() {
            return None;
        }
        let target = self.graph.edges[self.ei.usize()];

        // if would not be sufficient because some nodes may not have any edges.
        while self.ei >= self.graph.edge_start(self.ni + 1) {
            self.ni += 1;
        }
        let source = self.ni;

        self.ei += 1;
        Some((source, target))
    }
}
