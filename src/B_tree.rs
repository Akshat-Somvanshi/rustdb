use std::{cmp::Ordering, vec};

pub static HEADER: u16 = 4;
pub const BTREE_PAGE_SIZE: usize = 4096;
pub const BTREE_MAX_KEY_SIZE: usize = 1000;
pub const BTREE_MAX_VAL_SIZE: usize = 3000;
pub const BNODE_LEAF: u16 = 2;
pub const BNODE_NODE: u16 = 1;
pub const BNODE_INVALID: u16 = 0;
pub struct BNode {
    pub data: Vec<u8>,
}
// impl Iterator for BNode {
//     type Item = Vec<u8>;
//     fn next(&mut self) -> Option<Self::Item> {
//         let data = self.data.clone();
//         return Some(data);
//     }
// }
impl BNode {
    pub fn new() -> BNode {
        let bnode = BNode {
            data: vec![0; BTREE_PAGE_SIZE],
        };
        return bnode;
    }
    pub fn btype(&self) -> u16 {
        return u16::from_ne_bytes(self.data[..2].try_into().unwrap());
    }
    pub fn nkeys(&self) -> u16 {
        return u16::from_ne_bytes(self.data[2..4].try_into().unwrap());
    }
    pub fn set_header(&mut self, btype: u16, nkeys: u16) {
        self.data[..2].copy_from_slice(&btype.to_ne_bytes());
        self.data[2..4].copy_from_slice(&nkeys.to_ne_bytes());
    }
    pub fn get_pointer(&self, index: u16) -> u64 {
        assert!(index < self.nkeys());
        let position: usize = HEADER as usize + 8 * index as usize;
        return u64::from_ne_bytes(self.data[position..position + 8].try_into().unwrap());
    }
    pub fn set_pointer(&mut self, index: u16, value: u64) {
        assert!(index < self.nkeys());
        let position: usize = HEADER as usize + 8 * index as usize;
        self.data[position..position + 8].copy_from_slice(&value.to_ne_bytes());
    }
    // structure of a node is |node type (2B)|number of keys(2B)|pointers(each pointer is 8B)|offsets(each offset is 2B)|key-value pairs
    pub fn offset_position(&self, index: u16) -> u16 {
        assert!(index <= self.nkeys());
        if index >= 1 {
            return HEADER + 8 * self.nkeys() + 2 * (index);
        } else {
            return HEADER + 8 * self.nkeys();
        }
    }
    pub fn get_offset(&self, index: u16) -> u16 {
        let offset_position = self.offset_position(index) as usize;
        // println!("offset-{offset_position}");
        return u16::from_ne_bytes(
            self.data[offset_position..offset_position + 2]
                .try_into()
                .unwrap(),
        );
    }
    pub fn set_offset(&mut self, index: u16, offset: u16) {
        if index < self.nkeys() {
            let position = self.offset_position(index) as usize;
            // println!("{index}");
            // println!("{position}");
            println!("setting_offset:{offset}");
            self.data[position..position + 2].copy_from_slice(&offset.to_ne_bytes());
        }
    }
    pub fn kvpos(&self, index: u16) -> u16 {
        println!("index:{index}");
        assert!(index <= self.nkeys());
        println!("offset_value-{}", self.get_offset(index));
        return HEADER + self.nkeys() * 2 + self.nkeys() * 8 + self.get_offset(index);
    }
    pub fn get_key(&self, index: u16) -> Vec<u8> {
        assert!(index <= self.nkeys());
        let key_pos = self.kvpos(index) as usize;
        // println!("position:{key_pos}");
        let klen = u16::from_ne_bytes(self.data[key_pos..key_pos + 2].try_into().unwrap()) as usize;
        // key-value pair structure: |key_length(2B)|Value_length(2B)|key|value|
        return self.data[key_pos + 4..key_pos + 4 + klen].to_vec();
    }
    pub fn get_value(&self, index: u16) -> Vec<u8> {
        assert!(index <= self.nkeys());
        let key_pos = self.kvpos(index) as usize;
        let klen = u16::from_ne_bytes(self.data[key_pos..key_pos + 2].try_into().unwrap()) as usize;
        let vlen =
            u16::from_ne_bytes(self.data[key_pos + 2..key_pos + 4].try_into().unwrap()) as usize;
        return self.data[key_pos + 4 + klen..key_pos + 4 + klen + vlen].to_vec();
    }
    pub fn size(&self) -> u16 {
        let position = self.kvpos(self.nkeys() - 1);
        let last_index_containing_value = position as usize
            + self.get_key(self.nkeys() - 1).len()
            + self.get_value(self.nkeys() - 1).len();
        return last_index_containing_value as u16;
    }
    pub fn lookup_key(&self, key: &Vec<u8>) -> u16 {
        let mut found: u16 = 0;
        let mut i: u16 = 1;

        while i < self.nkeys() {
            println!("{:?}", self.get_key(i));
            if self.get_key(i).cmp(&vec![]) == Ordering::Equal {
                break;
            }
            if key.cmp(&self.get_key(i)) == Ordering::Equal
                || key.cmp(&self.get_key(i)) == Ordering::Greater
            {
                found = i;
            } else if key.cmp(&self.get_key(i)) == Ordering::Equal
                || key.cmp(&self.get_key(i)) == Ordering::Less
            {
                break;
            }
            i = i + 1;
        }
        return found;
    }
    pub fn node_append_kv_pair(&mut self, pointer: u64, index: u16, key: Vec<u8>, value: Vec<u8>) {
        self.set_pointer(index, pointer);
        // println!("{index}");
        let position = self.kvpos(index) as usize;
        // println!("position-{position}");
        self.data[position..position + 2]
            .copy_from_slice(&(key.clone().len() as u16).to_ne_bytes());
        self.data[position + 2..position + 4]
            .copy_from_slice(&(value.clone().len() as u16).to_ne_bytes());
        self.data[position + 4..position + 4 + key.len()].copy_from_slice(&key);
        self.data[position + 4 + key.len()..position + 4 + key.len() + value.len()]
            .copy_from_slice(&value);
        let offset_value = self.get_offset(index) + HEADER + key.len() as u16 + value.len() as u16;

        self.set_offset(index + 1, offset_value);
    }
}
pub fn leaf_insert(
    old_leaf_node: &BNode,
    new_leaf_node: &mut BNode,
    index: u16,
    key: Vec<u8>,
    value: Vec<u8>,
) {
    new_leaf_node.set_header(BNODE_LEAF, old_leaf_node.nkeys() + 1);
    println!("{index}");
    node_append_range(old_leaf_node, new_leaf_node, 0, 0, index);
    new_leaf_node.node_append_kv_pair(0, index, key, value);
    // println!("{:?}", new_leaf_node.get_key(1).len());
    // println!("{:?}", new_leaf_node.data);
    node_append_range(
        old_leaf_node,
        new_leaf_node,
        index + 1,
        index,
        old_leaf_node.nkeys() - index,
    );
    // println!("here");
    // println!("{:?}", new_leaf_node.data);
}
pub fn leaf_update(
    old_leaf_node: &BNode,
    new_leaf_node: &mut BNode,
    index: u16,
    key: Vec<u8>,
    value: Vec<u8>,
) {
    println!("{index}");
    new_leaf_node.set_header(BNODE_LEAF, old_leaf_node.nkeys() + 1);

    node_append_range(old_leaf_node, new_leaf_node, 0, 0, index);
    new_leaf_node.node_append_kv_pair(0, index, key, value);
    node_append_range(
        old_leaf_node,
        new_leaf_node,
        index + 1,
        index + 1,
        old_leaf_node.nkeys() - index,
    );
}
pub fn node_append_range(
    old_leaf_node: &BNode,
    new_leaf_node: &mut BNode,
    destination_new: u16,
    source_old: u16,
    range_size: u16,
) {
    assert!(destination_new + range_size <= new_leaf_node.nkeys());
    if source_old + range_size > old_leaf_node.nkeys() {
        println!("{source_old} + {range_size}");
        return;
    }
    if range_size == 0 {
        return;
    }
    for i in 0..range_size {
        new_leaf_node.set_pointer(
            destination_new + i,
            old_leaf_node.get_pointer(source_old + i),
        )
    }

    let new_offset_start = new_leaf_node.get_offset(destination_new);
    println!("new_offset_start:{new_offset_start}");

    let old_offset_start = old_leaf_node.get_offset(source_old);
    println!("{old_offset_start}");
    for i in 0..range_size {
        let offset_value =
            new_offset_start + old_leaf_node.get_offset(source_old + i) - old_offset_start;
        println!("offset:{offset_value}");

        new_leaf_node.set_offset(destination_new + i, offset_value);
    }
    // println!("{:?}", old_leaf_node.get_key(source_old));
    let begin = old_leaf_node.kvpos(source_old) as usize;
    let mut end: usize = 0;
    let index = source_old + range_size;

    if index == old_leaf_node.nkeys() {
        end = old_leaf_node.kvpos(index - 1) as usize
            + HEADER as usize
            + old_leaf_node.get_key(index - 1).len()
            + old_leaf_node.get_value(index - 1).len();
    } else {
        end = old_leaf_node.kvpos(index) as usize;
    }
    println!("{begin}-{end}");
    let destination_begin = new_leaf_node.kvpos(destination_new) as usize;
    let destination_end = destination_begin + end - begin;
    println!("{destination_begin}-{destination_end}");
    // let size = end - begin;
    // let mut slice = vec![0; size];

    // println!("{:?}", old_leaf_node.data);
    // slice.copy_from_slice(&old_leaf_node.data[begin..end]);
    // println!("{:?}", slice);
    new_leaf_node.data[destination_begin..destination_end]
        .copy_from_slice(&old_leaf_node.data[begin..end]);
    // println!("{:?}", new_leaf_node.data[begin..end].to_vec());
    let index = destination_new + range_size;
    let offset_value = new_leaf_node.get_offset(index - 1)
        + HEADER
        + new_leaf_node.get_key(index - 1).len() as u16
        + new_leaf_node.get_value(index - 1).len() as u16;
    new_leaf_node.set_offset(index, offset_value);
}
pub struct BTree {
    pub root: u64,
}
pub fn del(pointer: u64) {
    unsafe {
        let ptr = pointer as *mut Vec<u8>;
        let _ = Box::from_raw(ptr);
    }
}
pub fn new(node: BNode) -> u64 {
    let data_on_heap = Box::new(node.data);
    let pointer = Box::into_raw(data_on_heap);
    return pointer as u64;
}
pub fn get(pointer: u64) -> BNode {
    unsafe {
        let ptr = pointer as *mut Vec<u8>;
        let data = Box::from_raw(ptr);
        let result = *data.clone();
        Box::into_raw(data);
        return BNode { data: result };
    }
}
impl BTree {
    pub fn new() -> BTree {
        return BTree { root: 0 };
    }
    pub fn node_replace_kidN(
        &mut self,
        new_node: &mut BNode,
        index: u16,
        old_node: &BNode,
        kids: Vec<BNode>,
    ) {
        let inc = kids.len();
        new_node.set_header(BNODE_NODE, old_node.nkeys() + inc as u16 - 1);
        node_append_range(old_node, new_node, 0, 0, index);
        let mut i: u16 = 0;
        for node in kids {
            let temp = node.data.clone();
            let temp_node = BNode { data: temp };
            new_node.node_append_kv_pair(new(node), index + i, temp_node.get_key(0), vec![]);
            i = i + 1;
        }

        node_append_range(
            old_node,
            new_node,
            index + inc as u16,
            index + 1,
            old_node.nkeys() - index - 1,
        )
    }
    pub fn tree_insert(&mut self, node: BNode, key: Vec<u8>, value: Vec<u8>) -> BNode {
        let mut new = BNode {
            data: vec![0; 2 * BTREE_PAGE_SIZE],
        };
        let index = node.lookup_key(&key);
        println!("during insertion in tree:{index}");
        match node.btype() {
            BNODE_LEAF => {
                if key.cmp(&node.get_key(index)) == Ordering::Equal {
                    println!("voila, reached leaf-update");
                    leaf_update(&node, &mut new, index, key, value)
                } else {
                    println!("here inside leaf-insert");
                    leaf_insert(&node, &mut new, index + 1, key, value)
                }
            }
            BNODE_NODE => {
                self.node_insert(&node, &mut new, index, key, value);
            }
            _ => {
                println!("bad node!")
            }
        }
        // println!("{:?}", new.data);
        return new;
    }
    pub fn node_insert(
        &mut self,
        old_node: &BNode,
        new_node: &mut BNode,
        index: u16,
        key: Vec<u8>,
        value: Vec<u8>,
    ) {
        let kptr = old_node.get_pointer(index);
        let knode = self.tree_insert(get(kptr), key, value);
        del(kptr);
        let split = node_split3(knode);
        self.node_replace_kidN(new_node, index, old_node, split)
    }
    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        assert!(key.len() != 0 && key.len() <= BTREE_MAX_KEY_SIZE);
        assert!(value.len() <= BTREE_MAX_VAL_SIZE);
        if self.root == 0 {
            let mut root = BNode::new();
            root.set_header(BNODE_LEAF, 2);
            root.node_append_kv_pair(0, 0, vec![], vec![]);
            root.node_append_kv_pair(0, 1, key, value);
            self.root = new(root);
            return;
        } else {
            let node = self.tree_insert(get(self.root), key, value);
            // println!("{:?}", node.data);
            let nodes = node_split3(node);
            del(self.root);
            if nodes.len() > 1 {
                let mut root = BNode::new();
                root.set_header(BNODE_NODE, 2);
                let mut i = 0;
                for node in nodes {
                    println!("{:?}", node.data);
                    let key = node.get_key(0);
                    let pointer = new(node);
                    root.node_append_kv_pair(pointer, i, key, vec![]);
                    i = i + 1;
                }
                self.root = new(root);
            } else {
                let data = nodes[0].data.clone();
                self.root = new(BNode { data });
            }
        }
    }
    pub fn leaf_delete(&mut self, new_leaf_node: &mut BNode, old_leaf_node: &BNode, index: u16) {
        println!("leaf delete gets called");
        println!("old_leaf data: {:?}", old_leaf_node.data);
        new_leaf_node.set_header(old_leaf_node.btype(), old_leaf_node.nkeys() - 1);
        node_append_range(old_leaf_node, new_leaf_node, 0, 0, index);
        println!(
            "index:{}  remaining:{}",
            index,
            old_leaf_node.nkeys() - index - 1
        );
        node_append_range(
            old_leaf_node,
            new_leaf_node,
            index,
            index + 1,
            old_leaf_node.nkeys() - index - 1,
        );
        // println!("updated datata after deletion:{:?}", new_leaf_node.data);
    }
    pub fn node_merge(&mut self, left_node: &BNode, right_node: &BNode, new_node: &mut BNode) {
        println!("merging nodes");
        new_node.set_header(left_node.btype(), left_node.nkeys() + right_node.nkeys());
        node_append_range(left_node, new_node, 0, 0, left_node.nkeys());
        node_append_range(
            right_node,
            new_node,
            left_node.nkeys(),
            0,
            right_node.nkeys(),
        );
        println!("{:?}", new_node.data);
    }
    pub fn tree_delete(&mut self, node: &mut BNode, key: Vec<u8>) -> BNode {
        let index = node.lookup_key(&key);
        println!("inside_delete");
        match node.btype() {
            BNODE_LEAF => {
                if !(key.cmp(&node.get_key(index)) == Ordering::Equal) {
                    return BNode::new();
                }
                let mut new_node = BNode::new();
                self.leaf_delete(&mut new_node, node, index);
                // println!("new node data: {:?}", new_node.data);
                return new_node;
            }
            BNODE_NODE => {
                println!("bnode");
                return self.node_delete(node, index, key);
            }
            _ => {
                return BNode::new();
            }
        }
    }
    pub fn delete(&mut self, key: Vec<u8>) -> bool {
        assert!(key.len() != 0 && key.len() <= BTREE_MAX_KEY_SIZE);
        if self.root == 0 {
            return false;
        }
        // println!("key ot be deleted:{:?}", key);
        let updated_node = self.tree_delete(&mut get(self.root), key);
        if updated_node.data.len() == 0 {
            return false;
        }
        del(self.root);

        if updated_node.btype() == BNODE_NODE && updated_node.nkeys() == 1 {
            self.root = updated_node.get_pointer(0);
        } else {
            self.root = new(updated_node);
        }
        return true;
    }
    pub fn node_delete(&mut self, node: &mut BNode, index: u16, key: Vec<u8>) -> BNode {
        println!("inside_node_delete");
        let pointer = node.get_pointer(index);
        let mut updated_node = self.tree_delete(&mut get(pointer), key);
        if updated_node.data.len() == 0 {
            return BNode::new();
        }
        println!("{:?}", updated_node.data);
        del(pointer);
        let mut new_node = BNode::new();
        println!("{index}");
        let (merge_dir, sibling) = self.should_merge(&mut updated_node, node, index);
        println!("merge_direction:{merge_dir}");
        if merge_dir < 0 {
            let mut merged = BNode::new();
            self.node_merge(&sibling, &updated_node, &mut merged);
            del(node.get_pointer(index - 1));
            let key = merged.get_key(0);
            let pointer = new(merged);
            BTree::node_replace_kid2(&mut new_node, node, index - 1, pointer, key)
        }
        if merge_dir > 0 {
            let mut merged = BNode::new();
            self.node_merge(&updated_node, &sibling, &mut merged);
            del(node.get_pointer(index + 1));
            let key = merged.get_key(0);
            let pointer = new(merged);
            BTree::node_replace_kid2(&mut new_node, node, index - 1, pointer, key)
        }
        if merge_dir == 0 {
            assert!(updated_node.nkeys() > 0);
            println!("shouldn't merge");
            self.node_replace_kidN(&mut new_node, index, &node, vec![updated_node]);
        }
        return new_node;
    }
    pub fn node_replace_kid2(
        new_node: &mut BNode,
        old_node: &BNode,
        index: u16,
        pointer: u64,
        key: Vec<u8>,
    ) {
        new_node.set_header(old_node.btype(), old_node.nkeys() - 1);
        node_append_range(old_node, new_node, 0, 0, index);
        // [2,4,5,6,10,12,14]-new_node key
        // [2,4,5,6,7,10,12,14]-old_node key
        // so offset of 10 in new node would be same as offset of 7 in old.
        // node_append_range will calculate correct offset till 6 but after that it might copy the offset of old key with same value but since in old_node the key is placed further,
        // the offset might be incorrect
        // so if we get offset of 10 i.e 4 we are using index as offset here,in old_node for 10 it is 5 => so 4 and 5 are startinf offsets in new and old nodes respectively.
        // so if we are to calculate offset of 12 in new node it would be- new_node offset start + old_node offset for 12 -  old_node offset start
        // => 4 + 6 - 5 = 5, this is correct offset. So our earlier concern might have been useless.
        new_node.node_append_kv_pair(pointer, index, key, vec![]);
        // for i in index + 1..old_node.nkeys() {
        //     let offset_value = old_node.get_offset(i);
        // }
        node_append_range(
            old_node,
            new_node,
            index + 1,
            index + 2,
            old_node.nkeys() - index - 2,
        )
    }

    pub fn should_merge(
        &mut self,
        updated_node: &mut BNode,
        old_node: &BNode,
        index: u16,
    ) -> (i8, BNode) {
        if BTREE_PAGE_SIZE / 4 < updated_node.size() as usize {
            println!("size of updated_node:{:?}", updated_node.size());
            return (0, BNode::new());
        }
        if index > 0 {
            let sibling = get(old_node.get_pointer(index - 1));
            let size = sibling.size() + updated_node.size() - HEADER;
            println!("sibling size:{size}");
            if BTREE_PAGE_SIZE >= size as usize {
                return (-1, sibling);
            }
        }
        if index + 1 < old_node.nkeys() {
            let sibling = get(old_node.get_pointer(index + 1));
            let size = sibling.size() + updated_node.size() - HEADER;
            if BTREE_PAGE_SIZE >= size as usize {
                return (1, sibling);
            }
        }
        return (0, BNode::new());
    }
    pub fn search(&mut self, key: &Vec<u8>) -> (bool, u16, BNode) {
        let root_node = get(self.root);
        let mut found = false;
        let index = root_node.lookup_key(&key);
        println!("indexo:{index}");
        match root_node.btype() {
            BNODE_LEAF => {
                let index = root_node.lookup_key(key);
                if root_node.get_key(index).cmp(&key) == Ordering::Equal {
                    found = true;
                }
                return (found, index, root_node);
            }
            BNODE_NODE => {
                let mut node = get(root_node.get_pointer(index));
                while node.btype() != BNODE_LEAF {
                    let index = node.lookup_key(&key);
                    node = get(node.get_pointer(index));
                }
                let index = node.lookup_key(&key);
                if node.get_key(index).cmp(&key) == Ordering::Equal {
                    found = true;
                }
                return (found, index, node);
            }
            _ => {
                return (false, 0, BNode::new());
            }
        }
    }
}
pub fn node_split2(old_node: &BNode, left_node: &mut BNode, right_node: &mut BNode) {
    // println!("konichiwa");
    let split_index = old_node.nkeys() / 2;
    // println!("{split_index}");

    left_node.set_header(old_node.btype(), split_index);
    right_node.set_header(old_node.btype(), old_node.nkeys() - split_index);
    node_append_range(old_node, left_node, 0, 0, split_index);
    // println!("append to left_child done");
    // println!("{:?}", left_node.data);
    node_append_range(
        old_node,
        right_node,
        0,
        split_index,
        old_node.nkeys() - split_index,
    );
}
pub fn node_split3(old_node: BNode) -> Vec<BNode> {
    if BTREE_PAGE_SIZE > old_node.size() as usize {
        // println!("{:?}", old_node.size());
        // println!("here");
        // println!("{:?}", old_node.data);
        let mut modified_node = BNode::new();
        // println!("size less than page size");
        // println!("size-{}", old_node.size());
        modified_node
            .data
            .copy_from_slice(&old_node.data[..BTREE_PAGE_SIZE]);
        // println!("{}", modified_node.size());
        return vec![modified_node];
    } else {
        let mut left_node = BNode {
            data: vec![0; 2 * BTREE_PAGE_SIZE],
        };
        let mut right_node = BNode::new();

        node_split2(&old_node, &mut left_node, &mut right_node);
        // println!("size-{}", right_node.size());
        // println!("size-{}", left_node.size());
        if BTREE_PAGE_SIZE > left_node.size() as usize {
            let mut modified_node = BNode::new();
            modified_node
                .data
                .copy_from_slice(&left_node.data[..BTREE_PAGE_SIZE]);
            return vec![modified_node, right_node];
        } else {
            let mut extreme_left_node = BNode::new();
            let mut middle_node = BNode::new();
            node_split2(&left_node, &mut extreme_left_node, &mut middle_node);
            return vec![extreme_left_node, middle_node, right_node];
        }
    }
}
// the test covers insertion,deletion,merging,splitting in the b+ tree
#[cfg(test)]
mod test {

    use std::clone;

    use super::*;

    #[test]
    fn btype_and_nkeys_conversion() {
        let mut bnode = BNode::new();
        let btype = BNODE_LEAF;
        let nkeys: u16 = 300;
        bnode.set_header(btype, nkeys);
        assert_eq!(bnode.btype(), btype);
        assert_eq!(bnode.nkeys(), nkeys);
    }

    #[test]
    fn set_and_get_pointer() {
        let val: u64 = 242425;
        let mut bnode = BNode::new();
        let index = 4;
        bnode.set_header(1, 5);
        bnode.set_pointer(index, val);
        assert_eq!(bnode.get_pointer(index), val);
    }
    #[test]
    fn set_and_get_offset() {
        // let offset = 287;
        let index = 24;
        let mut bnode = BNode::new();
        let btype = 1;
        let nkeys = 24;
        bnode.set_header(btype, nkeys);
        let offset = 0;
        // we have made this value as 0 because offset is used to go to the the key-value pair at the given index, here since there is not k-v pair offset is 0.Working of offset
        // with k-v pairs is checked in another test below.
        bnode.set_offset(index, offset);
        // println!("{}", bnode.size());
        assert_eq!(offset, bnode.get_offset(index));
    }
    #[test]
    fn checking_get_and_new_functions() {
        let mut data = vec![0; 4096];
        data[4003] = 255;
        let node = BNode { data };
        let pointer = new(BNode {
            data: node.data.clone(),
        });
        let temp_pointer = pointer.clone();
        let dereferenced_data = get(pointer);
        assert_eq!(node.data, dereferenced_data.data);
        let data_obtained_with_temp = get(temp_pointer);
        assert_eq!(node.data, data_obtained_with_temp.data);
    }
    #[test]
    fn bnode_btree_initialization_check() {
        let tree = BTree::new();
        let node = BNode::new();
        assert_eq!(tree.root, 0);
        assert_eq!(node.data.len(), BTREE_PAGE_SIZE);
    }
    #[test]
    fn looking_up_index_to_insert_key() {
        // println!("here");
        let mut node = BNode::new();
        node.set_header(BNODE_LEAF, 3);
        let key: Vec<u8> = vec![1, 2, 3];
        let val: Vec<u8> = vec![2, 34, 24];
        node.node_append_kv_pair(0, 1, key.clone(), val.clone());

        let index_returned = node.lookup_key(&key.clone());
        println!("{index_returned}");
        let new_key: Vec<u8> = vec![23, 1, 42];
        let index_for_insertion = node.lookup_key(&new_key);
        assert_eq!(index_for_insertion, index_returned);
    }
    #[test]
    fn checking_insert_function() {
        let mut tree = BTree::new();
        let key = vec![1, 2, 3];
        let value = vec![4, 5, 6];
        tree.insert(key.clone(), value.clone());
        println!("here");
        // assert_eq!(key, root_node.get_key(1));
        // assert_eq!(value, root_node.get_value(1));
        let new_key = vec![3, 53, 2];
        let new_val = vec![242, 55, 2];
        tree.insert(new_key.clone(), new_val.clone());
        let root_node = get(tree.root);
        // println!("{:?}", root_node.data);
        // println!("{:?}", root_node.get_value(0));
        assert_eq!(new_key, root_node.get_key(2));
        assert_eq!(new_val, root_node.get_value(2));
        let nkey = vec![1; 999];
        let nval = vec![2; 2999];
        tree.insert(nkey.clone(), nval.clone());
        let root_node = get(tree.root);
        // println!("{}", root_node.data.len());
        // println!("{}", root_node.size());
        // println!("{:?}", root_node.data);

        let index = root_node.lookup_key(&nkey);
        assert_eq!(nkey, root_node.get_key(index));
        assert_eq!(nval, root_node.get_value(index));
        // println!("done till this insertion");
        let new_key = vec![1; 100];
        let new_val = vec![32; 232];
        tree.insert(new_key.clone(), new_val.clone());
        let _root_node = get(tree.root);
        // println!("{:?}", root_node.data);
        let (found, index, node) = tree.search(&vec![3, 53, 2]);
        // println!("key_index:{index}");
        assert!(found);
        assert_eq!(vec![3, 53, 2], node.get_key(index));
        let new_key = vec![2; 45];
        let new_val = vec![3; 49];
        tree.insert(new_key.clone(), new_val.clone());
        let root_node = get(tree.root);
        let (found, index, node) = tree.search(&new_key);
        // println!("{:?}", node.data);
        // println!("{:?}", root_node.data);

        for i in 0..root_node.nkeys() {
            let child = get(root_node.get_pointer(i));
            println!("child {0} kv pairs: {1}", i, child.nkeys());
        }
        assert_eq!(new_key.clone(), node.get_key(index));
        tree.insert(new_key.clone(), nval.clone());
        let _root_node = get(tree.root);
        let (found, index, node) = tree.search(&new_key);
        assert!(found);
        // println!("{:?}", node.data);
        assert_eq!(nval, node.get_value(index));
    }
    #[test]
    fn checking_node_append_kv() {
        let mut node = BNode::new();
        node.set_header(BNODE_LEAF, 2);
        node.node_append_kv_pair(0, 0, vec![], vec![]);
        let key = vec![1, 2, 3];
        let value = vec![4, 5, 6];
        node.node_append_kv_pair(0, 1, key.clone(), value.clone());
        // println!("{:?}", node.data);
        assert_eq!(node.get_key(1), key);
        assert_eq!(value, node.get_value(1));

        let mut new_node = BNode::new();
        new_node.set_header(node.btype(), node.nkeys() + 1);
        node_append_range(&node, &mut new_node, 0, 0, node.nkeys());
        // println!("{:?}", new_node.get_key(0));
        // println!("{:?}", new_node.data);
        assert_eq!(node.get_key(1), new_node.get_key(1));
        let nkey = vec![3, 51, 2];
        let nval = vec![241, 55, 2];
        new_node.node_append_kv_pair(0, 2, nkey.clone(), nval.clone());
        assert_eq!(nkey, new_node.get_key(2));
        assert_eq!(nval, new_node.get_value(2));
    }
    #[test]
    fn checking_delete() {
        let mut tree = BTree::new();
        let key = vec![1, 2, 3];
        let value = vec![4, 5, 6];
        tree.insert(key.clone(), value.clone());
        let new_key = vec![3, 53, 2];
        let new_val = vec![242, 55, 2];
        tree.insert(new_key.clone(), new_val.clone());

        let nkey = vec![1; 999];
        let nval = vec![2; 2999];
        tree.insert(nkey.clone(), nval.clone());
        let root_node = get(tree.root);

        let index = root_node.lookup_key(&nkey);
        let new_key = vec![1; 100];
        let new_val = vec![32; 232];
        tree.insert(new_key.clone(), new_val.clone());
        let _root_node = get(tree.root);
        let (found, index, node) = tree.search(&vec![3, 53, 2]);
        assert_eq!(vec![3, 53, 2], node.get_key(index));
        let new_key = vec![2; 45];
        let new_val = vec![3; 49];
        tree.insert(new_key.clone(), new_val.clone());
        println!("safe_till_here");
        // tree.delete(key.clone());
        tree.delete(nkey.clone());
        let (found, index, node) = tree.search(&nkey);
        println!("{:?}", node.data);
        assert!(!found);
        assert_ne!(node.get_key(index), nkey);
        tree.delete(new_key.clone());
        let (found, index, node) = tree.search(&new_key);
        assert!(!found);
        // println!("{:?}", node.data);
        assert_ne!(node.get_key(index), nkey);
        tree.delete(vec![1; 100]);
        let (found, index, node) = tree.search(&vec![1; 100]);
        assert!(!found);
        // println!("{:?}", node.data);
        assert_ne!(node.get_key(index), vec![1; 100]);
        tree.delete(vec![3, 53, 2]);
        let (found, index, node) = tree.search(&vec![3, 53, 2]);
        assert!(!found);
        // println!("{:?}", node.data);
        assert_ne!(node.get_key(index), vec![3, 53, 2]);
        tree.delete(key.clone());
        let (found, index, node) = tree.search(&key.clone());
        assert!(!found);
        assert_ne!(node.get_key(index), key);
        let root_node = get(tree.root);
        println!("{:?}", root_node.data);
        // even after deleting all the keys we still have the vec![] which was inserted at the start of the insert operation
        assert!(root_node.nkeys() == 1);
    }
}
