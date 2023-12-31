use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::rc::{Rc, Weak};
use uuid::Uuid;

#[derive(Debug)]
pub struct BinaryTreeNode {
    pub id: Uuid,
    pub name: String,
    pub data: u32,
    pub parent: BinaryTreeNodeWeakRef,
    pub left: Option<BinaryTreeNodeRef>,
    pub right: Option<BinaryTreeNodeRef>,
}

pub type BinaryTreeNodeRef = Rc<RefCell<BinaryTreeNode>>;
pub type BinaryTreeNodeWeakRef = Weak<RefCell<BinaryTreeNode>>;

impl Ord for BinaryTreeNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.cmp(&other.data)
    }
}

impl PartialOrd for BinaryTreeNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BinaryTreeNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BinaryTreeNode {}

pub struct BinaryTree {
    pub root: Option<BinaryTreeNodeRef>,
}

impl BinaryTree {
    pub fn with_root(root: BinaryTreeNodeRef) -> Self {
        BinaryTree { root: Some(root) }
    }

    pub fn new_node() -> BinaryTreeNodeRef {
        Rc::new(RefCell::new(BinaryTreeNode {
            id: Uuid::new_v4(),
            name: "".to_string(),
            data: 0,
            parent: Weak::new(),
            left: None,
            right: None,
        }))
    }

    pub fn count(node: &BinaryTreeNodeRef) -> usize {
        let mut count = 0;
        let mut queue = VecDeque::new();
        queue.push_back(node.clone());
        while let Some(node) = queue.pop_front() {
            count += 1;

            let n = node.borrow();
            if let Some(left) = n.left.as_ref() {
                queue.push_back(left.clone());
            }
            if let Some(right) = n.right.as_ref() {
                queue.push_back(right.clone());
            }
        }
        count
    }

    pub fn flatten_top_down(node: BinaryTreeNodeRef) -> Vec<BinaryTreeNodeRef> {
        let mut nodes = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(node.clone());
        while let Some(node) = queue.pop_front() {
            nodes.push(node.clone());
            let n = node.borrow();
            if let Some(left) = n.left.as_ref() {
                queue.push_back(left.clone());
            }
            if let Some(right) = n.right.as_ref() {
                queue.push_back(right.clone());
            }
        }
        nodes
    }

    pub fn flatten_inorder(node_ref: BinaryTreeNodeRef) -> Vec<BinaryTreeNodeRef> {
        let mut root = Some(node_ref.clone());
        let mut nodes = VecDeque::new();

        let mut leftdone = false;
        while let Some(root_ref) = root.as_ref() {
            let mut current_ref = root_ref.clone();
            if !leftdone {
                if let Some(leftmost) = Self::leftmost(&current_ref) {
                    current_ref = leftmost.clone();
                }
            }

            leftdone = true;
            nodes.push_back(current_ref.clone());
            root = Some(current_ref.clone());

            let root_node = current_ref.borrow();

            if let Some(right) = root_node.right.as_ref() {
                leftdone = false;
                root = Some(right.clone());
            } else if let Some(parent) = root_node.parent.upgrade() {
                let mut root_parent = Some(parent.clone());
                let mut parent_right = parent.clone().borrow().right.clone();
                while root_parent.is_some() {
                    if !Self::is_same(&root, &parent_right) {
                        break;
                    }

                    root = root_parent;
                    root_parent = if root.is_some() {
                        root.clone().unwrap().borrow().parent.upgrade()
                    } else {
                        None
                    };

                    parent_right = if root_parent.is_some() {
                        root_parent.clone().unwrap().borrow().right.clone()
                    } else {
                        None
                    };
                }
                root = root_parent;
            } else {
                break;
            }
        }

        nodes.into()
    }

    pub fn get_root(node_ref: &BinaryTreeNodeRef) -> BinaryTreeNodeRef {
        let mut start = node_ref.clone();
        while let Some(parent) = start.clone().borrow().parent.upgrade() {
            start = parent.clone();
        }
        start
    }

    pub fn assign_parents(node: &BinaryTreeNodeRef) {
        let mut queue = VecDeque::new();
        queue.push_back(node.clone());
        while let Some(node) = queue.pop_front() {
            let n = node.borrow();
            if let Some(left) = n.left.as_ref() {
                left.borrow_mut().parent = Rc::downgrade(&node);
                queue.push_back(left.clone());
            }
            if let Some(right) = n.right.as_ref() {
                right.borrow_mut().parent = Rc::downgrade(&node);
                queue.push_back(right.clone());
            }
        }
    }

    pub fn leftmost(node_ref: &BinaryTreeNodeRef) -> Option<BinaryTreeNodeRef> {
        let mut leftmost = None;
        let mut current = node_ref.clone();
        loop {
            let current_ref = current;
            let node = current_ref.borrow();
            if let Some(left) = node.left.as_ref() {
                current = left.clone();
                leftmost = Some(current.clone());
            } else {
                return leftmost;
            }
        }
    }

    pub fn is_same(v1: &Option<BinaryTreeNodeRef>, v2: &Option<BinaryTreeNodeRef>) -> bool {
        Self::get_node_id(v1) == Self::get_node_id(v2)
    }

    fn get_node_id(v: &Option<BinaryTreeNodeRef>) -> Option<Uuid> {
        v.as_ref().map(|node| node.borrow().id)
    }

    pub fn invert_recursive(node_ref: &BinaryTreeNodeRef) {
        let mut node = node_ref.borrow_mut();

        if let Some(right) = &node.right {
            Self::invert_recursive(right);
        }
        if let Some(left) = &node.left {
            Self::invert_recursive(left);
        }

        // swap child nodes
        let tmp = node.right.take();
        node.right = node.left.take();
        node.left = tmp;
    }

    pub fn invert_iterative(root_ref: BinaryTreeNodeRef) {
        let mut queue = VecDeque::new();
        queue.push_back(root_ref);
        while let Some(node_ref) = queue.pop_front() {
            let mut node = node_ref.borrow_mut();
            if let Some(right) = &node.right {
                queue.push_back(right.clone());
            }
            if let Some(left) = &node.left {
                queue.push_back(left.clone());
            }

            // swap child nodes
            let tmp = node.right.take();
            node.right = node.left.take();
            node.left = tmp;
        }
    }
}

pub mod utils {

    use super::*;
    pub const NODES_COUNT: usize = 15;

    pub fn populate_node_list() -> Vec<BinaryTreeNodeRef> {
        let mut list = Vec::<BinaryTreeNodeRef>::with_capacity(NODES_COUNT);
        (0..NODES_COUNT).for_each(|n| {
            let node_ref = BinaryTree::new_node();
            node_ref.borrow_mut().name = format!("n{}", n);
            node_ref.borrow_mut().data = n as u32;
            list.push(node_ref)
        });
        list
    }

    pub fn populate_balanced_binary_tree() -> BinaryTreeNodeRef {
        /*
        node names:
                     n0
                /           \
              n1              n2
            /    \          /     \
          n3      n4       n5      n6
         /   \   /   \   /   \    /   \
        n7   n8 n9  n10 n11  n12 n13  n14

        left_child = parent * 2 + 1
        right_child = parent * 2 + 2 = left_child + 1
        parent_of_left_child = (left_child - 1)/2
        parent_of_right_child = (right_child - 2)/2

        position:
        is_left  = (n % 2) != 0
        is_rignt = (n % 2) == 0
        */

        let nodes = populate_node_list();
        (0..NODES_COUNT).for_each(|n| {
            let left_child = n * 2 + 1;
            if left_child < NODES_COUNT {
                nodes[n].borrow_mut().left = Some(nodes[left_child].clone());
            }
            let right_child = left_child + 1;
            if left_child < NODES_COUNT {
                nodes[n].borrow_mut().right = Some(nodes[right_child].clone());
            }
        });
        BinaryTree::assign_parents(&nodes[0]);
        nodes[0].clone()
    }

    pub fn populate_balanced_binary_search_tree() -> BinaryTreeNodeRef {
        /*
        node values:
                       8
                /             \
              4                12
            /    \           /    \
           2       6       10      14
         /   \   /  \     /  \    /  \
        1     3 5    7   9   11  13   15
        */

        let node_values = [8, 4, 12, 2, 6, 10, 14, 1, 3, 5, 7, 9, 11, 13, 15];

        let root = populate_balanced_binary_tree();
        let flatten = BinaryTree::flatten_top_down(root.clone());
        flatten.iter().enumerate().for_each(|v| {
            v.1.borrow_mut().data = node_values[v.0];
        });
        root
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::binary_tree::{utils::*, BinaryTree};

    #[test]
    fn populate_node_ref_list_test() {
        let list = populate_node_list();
        assert_eq!(list.len(), NODES_COUNT);
        let names: Vec<_> = list.iter().map(|v| v.borrow().name.clone()).collect();
        (0..NODES_COUNT).for_each(|n| {
            let name = format!("n{}", n);
            assert!(names.contains(&name));
        })
    }

    #[test]
    fn binary_tree_populate_test1() {
        let root = populate_balanced_binary_tree();
        assert_eq!(root.borrow().name, "n0".to_string());

        let n0 = root.borrow();
        assert_eq!(n0.parent.upgrade(), None);

        let n1 = n0.left.as_ref().unwrap().borrow();
        assert_eq!(n1.name, "n1".to_string());
        assert_eq!(n1.parent.upgrade().unwrap().borrow().name, "n0".to_string());

        let n2 = n0.right.as_ref().unwrap().borrow();
        assert_eq!(n2.name, "n2".to_string());
        assert_eq!(n2.parent.upgrade().unwrap().borrow().name, "n0".to_string());

        let n3 = n1.left.as_ref().unwrap().borrow();
        assert_eq!(n3.name, "n3".to_string());
        assert_eq!(n3.parent.upgrade().unwrap().borrow().name, "n1".to_string());

        let n4 = n1.right.as_ref().unwrap().borrow();
        assert_eq!(n4.name, "n4".to_string());
        assert_eq!(n4.parent.upgrade().unwrap().borrow().name, "n1".to_string());

        let n5 = n2.left.as_ref().unwrap().borrow();
        assert_eq!(n5.name, "n5".to_string());
        assert_eq!(n5.parent.upgrade().unwrap().borrow().name, "n2".to_string());

        let n6 = n2.right.as_ref().unwrap().borrow();
        assert_eq!(n6.name, "n6".to_string());
        assert_eq!(n6.parent.upgrade().unwrap().borrow().name, "n2".to_string());

        let n7 = n3.left.as_ref().unwrap().borrow();
        assert_eq!(n7.name, "n7".to_string());
        assert_eq!(n7.parent.upgrade().unwrap().borrow().name, "n3".to_string());

        let n8 = n3.right.as_ref().unwrap().borrow();
        assert_eq!(n8.name, "n8".to_string());
        assert_eq!(n8.parent.upgrade().unwrap().borrow().name, "n3".to_string());

        let n9 = n4.left.as_ref().unwrap().borrow();
        assert_eq!(n9.name, "n9".to_string());
        assert_eq!(n9.parent.upgrade().unwrap().borrow().name, "n4".to_string());

        let n10 = n4.right.as_ref().unwrap().borrow();
        assert_eq!(n10.name, "n10".to_string());
        assert_eq!(
            n10.parent.upgrade().unwrap().borrow().name,
            "n4".to_string()
        );

        let n11 = n5.left.as_ref().unwrap().borrow();
        assert_eq!(n11.name, "n11".to_string());
        assert_eq!(
            n11.parent.upgrade().unwrap().borrow().name,
            "n5".to_string()
        );

        let n12 = n5.right.as_ref().unwrap().borrow();
        assert_eq!(n12.name, "n12".to_string());
        assert_eq!(
            n12.parent.upgrade().unwrap().borrow().name,
            "n5".to_string()
        );

        let n13 = n6.left.as_ref().unwrap().borrow();
        assert_eq!(n13.name, "n13".to_string());
        assert_eq!(
            n13.parent.upgrade().unwrap().borrow().name,
            "n6".to_string()
        );

        let n14 = n6.right.as_ref().unwrap().borrow();
        assert_eq!(n14.name, "n14".to_string());
        assert_eq!(
            n14.parent.upgrade().unwrap().borrow().name,
            "n6".to_string()
        );
    }

    #[test]
    fn binary_tree_populate_test2() {
        let root = populate_balanced_binary_tree();
        let nodes = BinaryTree::flatten_top_down(root);
        let nodes_count = nodes.len();
        assert_eq!(nodes_count, NODES_COUNT);

        for (index, node_ref) in nodes.iter().enumerate() {
            let node = node_ref.borrow();
            assert_eq!(node.name, format!("n{index}"));
            if index == 0 {
                assert_eq!(node.parent.upgrade(), None);
            } else {
                let parent_position = if index % 2 != 0 { 1 } else { 2 };
                let parent = (index - parent_position) / 2;
                assert_eq!(
                    node.parent.upgrade().unwrap().borrow().name,
                    format!("n{}", parent)
                );
            }
            let left = index * 2 + 1;
            if left < nodes_count {
                assert_eq!(
                    node.left.as_ref().unwrap().borrow().name,
                    format!("n{}", left)
                );
            }
            let right = left + 1;
            if left < nodes_count {
                assert_eq!(
                    node.right.as_ref().unwrap().borrow().name,
                    format!("n{}", right)
                );
            }
        }
    }

    #[test]
    fn count() {
        let node = populate_balanced_binary_tree();
        assert_eq!(BinaryTree::count(&node), NODES_COUNT);
    }

    #[test]
    fn flatten_top_down() {
        let root = populate_balanced_binary_search_tree();

        let flatten_nodes = BinaryTree::flatten_top_down(root);
        assert_eq!(flatten_nodes.len(), NODES_COUNT);

        let flatten_names: Vec<_> = flatten_nodes
            .iter()
            .map(|n| n.borrow().name.clone())
            .collect();

        let expected_names = (0..NODES_COUNT)
            .map(|n| format!("n{}", n))
            .collect::<Vec<_>>();
        assert_eq!(flatten_names, expected_names);
    }

    #[test]
    fn leftmost() {
        let expected = HashMap::from([
            ("n0", Some("n7")),
            ("n1", Some("n7")),
            ("n2", Some("n11")),
            ("n3", Some("n7")),
            ("n4", Some("n9")),
            ("n5", Some("n11")),
            ("n6", Some("n13")),
            ("n7", None),
            ("n8", None),
            ("n9", None),
            ("n10", None),
            ("n11", None),
            ("n12", None),
            ("n13", None),
            ("n14", None),
        ]);

        let root = populate_balanced_binary_tree();
        let flatten_nodes = BinaryTree::flatten_top_down(root);
        assert_eq!(flatten_nodes.len(), expected.len());
        let mut flatten_names: Vec<_> = flatten_nodes
            .iter()
            .map(|n| n.borrow().name.clone())
            .collect();

        let mut expected_names = expected.iter().map(|v| v.0.to_string()).collect::<Vec<_>>();
        flatten_names.sort();
        expected_names.sort();
        assert_eq!(flatten_names, expected_names);

        for node_ref in flatten_nodes {
            let node = node_ref.borrow();
            let leftmost = BinaryTree::leftmost(&node_ref);
            let name;
            assert_eq!(
                expected[node.name.as_str()],
                match leftmost {
                    Some(left_ref) => {
                        let node = left_ref.borrow();
                        name = node.name.clone();
                        Some(name.as_str())
                    }
                    None => None,
                }
            );
        }
    }

    #[test]
    fn flatten_inorder() {
        let expected = [
            "n7", "n3", "n8", "n1", "n9", "n4", "n10", "n0", "n11", "n5", "n12", "n2", "n13", "n6",
            "n14",
        ];

        let root = populate_balanced_binary_tree();
        let flatten_nodes: Vec<_> = BinaryTree::flatten_inorder(root.clone());
        assert_eq!(flatten_nodes.len(), expected.len());

        let flatten_names: Vec<_> = flatten_nodes
            .iter()
            .map(|n| n.borrow().name.clone())
            .collect();
        assert_eq!(flatten_names, expected);
    }

    #[test]
    fn invert_recursive() {
        let expected = [
            "n14", "n6", "n13", "n2", "n12", "n5", "n11", "n0", "n10", "n4", "n9", "n1", "n8",
            "n3", "n7",
        ];

        let root = populate_balanced_binary_tree();
        BinaryTree::invert_recursive(&root);

        let flatten_nodes: Vec<_> = BinaryTree::flatten_inorder(root.clone());
        assert_eq!(flatten_nodes.len(), expected.len());

        let flatten_names: Vec<_> = flatten_nodes
            .iter()
            .map(|n| n.borrow().name.clone())
            .collect();
        assert_eq!(flatten_names, expected);
    }

    #[test]
    fn invert_iterative() {
        let expected = [
            "n14", "n6", "n13", "n2", "n12", "n5", "n11", "n0", "n10", "n4", "n9", "n1", "n8",
            "n3", "n7",
        ];

        let root = populate_balanced_binary_tree();
        BinaryTree::invert_iterative(root.clone());

        let flatten_nodes: Vec<_> = BinaryTree::flatten_inorder(root.clone());
        assert_eq!(flatten_nodes.len(), expected.len());

        let flatten_names: Vec<_> = flatten_nodes
            .iter()
            .map(|n| n.borrow().name.clone())
            .collect();
        assert_eq!(flatten_names, expected);
    }
}
