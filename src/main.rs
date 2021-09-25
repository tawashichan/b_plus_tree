use std::{collections::HashMap, vec};
use thiserror::Error;

type Key = String;

type NodeId = usize;

type NodeMap = HashMap<NodeId, Node>;

#[derive(Debug, Clone)]
pub enum Node {
    // 右端はmaxのnodeを指す,中央のvecは左側のnodeのpointer意味する
    Mid(NodeId, Vec<Key>, Vec<NodeId>),
    Leaf(NodeId, Vec<Key>, Option<NodeId>),
}

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("key_not_found")]
    KeyNotFound,
}

#[derive(Debug)]
pub struct BTree {
    root_id: NodeId,
    node_map: NodeMap,
    node_key_num: usize,
    node_id_generator: NodeIdGenerator,
}

#[derive(Debug)]
struct NodeIdGenerator {
    node_counter: usize,
}

impl NodeIdGenerator {
    fn generate_node_id(&mut self) -> NodeId {
        let id = self.node_counter;
        self.node_counter += 1;
        id
    }
}

impl BTree {
    fn new(node_key_num: usize) -> Self {
        let mut node_id_generator = NodeIdGenerator { node_counter: 0 };
        let root_index = node_id_generator.generate_node_id();

        let node = Node::Leaf(root_index, vec![], None);
        let mut node_map = HashMap::new();
        node_map.insert(root_index, node);

        BTree {
            root_id: root_index,
            node_map: node_map.clone(),
            node_key_num,
            node_id_generator: node_id_generator,
        }
    }

    fn insert(&mut self, key: &String, value: &String) -> Result<(), Error> {
        let insert_result = self.insert_rec(self.root_id, key, value)?;
        match insert_result {
            Some((up_key, right_node_id)) => {
                let new_node_id = self.node_id_generator.generate_node_id();
                let new_node =
                    Node::Mid(new_node_id, vec![up_key], vec![self.root_id, right_node_id]);
                self.root_id = new_node_id;
                self.node_map.insert(new_node_id, new_node);
                Ok(())
            }
            None => Ok(()),
        }
    }

    fn insert_rec(
        &mut self,
        current_node_id: NodeId,
        key: &String,
        value: &String,
    ) -> Result<Option<(Key, NodeId)>, Error> {
        println!("node_id: {:?} self:{:?}", current_node_id, self);

        let current_node = self.node_map.get_mut(&current_node_id).unwrap();
        let insert_result = match current_node {
            Node::Leaf(id, keys, sibling) => {
                // todo 右端以外にinsertのパターンに対応
                keys.push(key.clone());

                if keys.len() < self.node_key_num {
                    None
                } else {
                    let new_node_keys = keys.split_off(self.node_key_num / 2);
                    let new_node_id = self.node_id_generator.generate_node_id();
                    *sibling = Some(new_node_id);

                    let min_key = new_node_keys.first().unwrap().clone();
                    let new_node = Node::Leaf(new_node_id, new_node_keys, None);

                    self.node_map.insert(new_node_id, new_node);

                    println!("insert {:?}", self);

                    Some((min_key, new_node_id))
                }
            }
            Node::Mid(_, keys, node_ids) => {
                // やばい
                let target_node_id = {
                    let mut node_id = None;
                    for (i, k) in keys.iter().enumerate() {
                        if key <= k {
                            node_id = Some(node_ids[i]);
                        }
                    }
                    if let Some(node_id) = node_id {
                        node_id
                    } else {
                        node_ids[node_ids.len() - 1]
                    }
                };
                self.insert_rec(target_node_id, key, value)?
            }
        };

        match insert_result {
            Some((min_key, right_node_id)) => {
                let current_node = self.node_map.get_mut(&current_node_id).unwrap();
                match current_node {
                    Node::Leaf(_, _, _) => Ok(Some((min_key, right_node_id))),
                    Node::Mid(_, keys, node_ids) => {
                        println!("min_key: {:?} right_node_id: {:?}", min_key, right_node_id);
                        // todo  右端への追加でないパターンに対応
                        keys.push(min_key);
                        node_ids.push(right_node_id);

                        if keys.len() < self.node_key_num {
                            Ok(None)
                        } else {
                            // 葉ノードを二つに分割し,右側の最小値を親ノードに持っていく
                            let mut new_node_keys_tmp = keys.split_off(self.node_key_num / 2);
                            let new_node_keys = new_node_keys_tmp.split_off(1);
                            let up_key = new_node_keys_tmp[0].clone();

                            let new_node_ids = node_ids.split_off(self.node_key_num / 2 + 1);

                            let new_node_id = self.node_id_generator.generate_node_id();

                            let new_node = Node::Mid(new_node_id, new_node_keys, new_node_ids);
                            println!("new_node: {:?}", new_node);
                            self.node_map.insert(new_node_id, new_node);

                            println!("mid splitted: {:?},current: {:?}", self, current_node_id);
                            Ok(Some((up_key, new_node_id)))
                        }
                    }
                }
            }
            None => Ok(None),
        }
    }
}

fn main() {
    let mut t = BTree::new(3);
    t.insert(&"1".to_string(), &"1".to_string());
    t.insert(&"3".to_string(), &"3".to_string());

    println!("\n insert 5");
    t.insert(&"5".to_string(), &"5".to_string());

    println!("\n insert 7");
    t.insert(&"7".to_string(), &"7".to_string());

    println!("\n insert 9");
    t.insert(&"9".to_string(), &"9".to_string());
    println!("{:?}", t);
}
