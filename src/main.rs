use rustdb::B_tree::{get, new, node_append_range, BNode, BTree, BNODE_LEAF};

fn main() {
    // let mut tree = BTree::new();
    // let key = vec![1, 2, 3];
    // let value = vec![4, 5, 6];
    // tree.insert(key.clone(), value.clone());
    // let new_key = vec![3, 53, 2];
    // let new_val = vec![242, 55, 2];
    // tree.insert(new_key.clone(), new_val.clone());
    // let root_node = get(tree.root);

    // let nkey = vec![1; 999];
    // let nval = vec![2; 2999];
    // tree.insert(nkey.clone(), nval.clone());
    // let root_node = get(tree.root);

    // let index = root_node.lookup_key(&nkey);
    // let new_key = vec![1; 100];
    // let new_val = vec![32; 232];
    // tree.insert(new_key.clone(), new_val.clone());
    // let _root_node = get(tree.root);
    // let (found, index, node) = tree.search(&vec![3, 53, 2]);
    // assert_eq!(vec![3, 53, 2], node.get_key(index));
    // let new_key = vec![2; 45];
    // let new_val = vec![3; 49];
    // tree.insert(new_key.clone(), new_val.clone());
    // println!("safe_till_here");
    // tree.delete(nkey.clone());
    // let (found, index, node) = tree.search(&nkey);
    // println!("{:?}", node.data);

    // assert_ne!(node.get_key(index), nkey);
    // let done = tree.delete(new_key.clone());
    // println!("{done}");
    // let root_node = get(tree.root);
    // // println!("{:?}", root_node.data);
    // let (found, index, node) = tree.search(&new_key);
}
