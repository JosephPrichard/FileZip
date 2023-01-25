use std::cmp::Ordering;

pub struct Tree {
    pub root: Box<Node>,
    pub symbol_count: u32
}

pub struct Node {
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
    pub plain_symbol: u8,
    pub weight: u64
}

impl Node {
    // creates a leaf structure with no children
    pub fn leaf(symbol: u8, weight: u64) -> Node {
        Node {
            left: None,
            right: None,
            plain_symbol: symbol,
            weight
        }
    }

    // moves the left and right nodes
    pub fn internal(left: Box<Node>, right: Box<Node>, symbol: u8, weight: u64) -> Node {
        Node {
            left: Some(Box::new(*left)),
            right: Some(Box::new(*right)),
            plain_symbol: symbol,
            weight
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left == None && self.right == None
    }
}

impl Eq for Node {}

impl PartialEq<Self> for Node {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl PartialOrd<Self> for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.weight.cmp(&self.weight)
    }
}