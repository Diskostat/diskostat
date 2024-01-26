use super::model::{entry_node::EntryNode, entry_type::EntryType};

pub(crate) fn delete_entry(node: &EntryNode) -> std::io::Result<()> {
    match node.entry_type {
        EntryType::Directory => std::fs::remove_dir_all(node.path.clone()),
        EntryType::File(_) => std::fs::remove_file(node.path.clone()),
    }
}
