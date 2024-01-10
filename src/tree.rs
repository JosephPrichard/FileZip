// Joseph Prichard
// 1/5/2023
// Type definitions and utilities for the binary tree structure used for huffman coding

use std::cmp::Ordering;

pub struct Tree {
    pub left: Option<Box<Tree>>,
    pub right: Option<Box<Tree>>,
    pub plain_symbol: u8,
    pub weight: u64,
}

impl Tree {
    // creates a leaf structure with no children
    pub fn leaf(symbol: u8, weight: u64) -> Tree {
        Tree {
            left: None,
            right: None,
            plain_symbol: symbol,
            weight,
        }
    }

    // moves the left and right nodes
    pub fn internal(left: Box<Tree>, right: Box<Tree>, symbol: u8, weight: u64) -> Tree {
        Tree {
            left: Some(Box::new(*left)),
            right: Some(Box::new(*right)),
            plain_symbol: symbol,
            weight,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left == None && self.right == None
    }
}

impl Eq for Tree {}

impl PartialEq<Self> for Tree {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl PartialOrd<Self> for Tree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Tree {
    fn cmp(&self, other: &Self) -> Ordering {
        other.weight.cmp(&self.weight)
    }
}