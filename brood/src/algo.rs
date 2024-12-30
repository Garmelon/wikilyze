use std::{cmp::Reverse, collections::BinaryHeap};

use crate::graph::{EdgeIdx, Graph, NodeIdx};

pub struct Dijkstra<'a> {
    graph: &'a Graph,
    cost: Vec<u32>,
    pred: Vec<NodeIdx>,
}

impl<'a> Dijkstra<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            graph,
            cost: vec![u32::MAX; graph.nodes.len()],
            pred: vec![NodeIdx::NONE; graph.nodes.len()],
        }
    }

    pub fn run(
        &mut self,
        start: NodeIdx,
        goal: impl Fn(NodeIdx) -> bool,
        cost: impl Fn(NodeIdx, EdgeIdx, NodeIdx) -> u32,
    ) {
        self.cost[start.usize()] = 0;
        let mut queue = BinaryHeap::new();
        queue.push((Reverse(0), start));

        while let Some((Reverse(curr_cost), curr)) = queue.pop() {
            if goal(curr) {
                break; // We've found the shortest path to our target
            }

            // These seem to never actually occur
            // if curr_cost > self.cost[curr.usize()] {
            //     continue; // Outdated entry
            // }

            for edge in self.graph.edge_range(curr).map(EdgeIdx::new) {
                let next = self.graph.edges[edge.usize()];
                let next_cost = curr_cost + cost(curr, edge, next);
                if next_cost < self.cost[next.usize()] {
                    self.cost[next.usize()] = next_cost;
                    self.pred[next.usize()] = curr;
                    queue.push((Reverse(next_cost), next));
                }
            }
        }
    }

    #[inline]
    pub fn cost(&self, node: NodeIdx) -> u32 {
        self.cost[node.usize()]
    }

    #[inline]
    pub fn pred(&self, node: NodeIdx) -> NodeIdx {
        self.pred[node.usize()]
    }

    pub fn path(&self, goal: NodeIdx) -> Vec<NodeIdx> {
        let mut path = vec![];
        let mut at = goal;

        loop {
            path.push(at);
            at = self.pred(at);
            if at == NodeIdx::NONE {
                break;
            }
        }

        path.reverse();
        path
    }
}
