mod circle_buffer;
pub use circle_buffer::CircleBuffer;


/// Representation of a Node's id.
pub type Id = usize;

/// A node in a Hierarchy. May contain children, have a parent, and stores some sort of item.
pub struct Node<T> {
    parent: Option<Id>,
    children: Vec<Id>,
    item: T,
}

impl<T> Node<T>
where
    T: Sized,
{
    /// Creates a new node
    fn new(item: T) -> Self {
        Self {
            item,
            parent: None,
            children: vec![],
        }
    }

    /// Returns a mutable reference to the item.
    pub fn item_mut(&mut self) -> &mut T {
        &mut self.item
    }

    /// Returns the parent of the node.
    pub fn parent(&self) -> Option<Id> {
        self.parent
    }

    /// Returns the children of the node.
    pub fn children(&self) -> Vec<Id> {
        self.children.clone()
    }

    /// Returns whether children exist for the node.
    pub fn has_children(&self) -> bool {
        self.children.is_empty() == false
    }

    /// Removes a child.
    fn remove_child(&mut self, id: Id) {
        let mut child_index = None;
        for (index, child) in self.children.iter().enumerate() {
            if *child == id {
                child_index = Some(index);
                break;
            }
        }

        if let Some(child_index) = child_index {
            self.children.remove(child_index);
        }
    }
}

// TODO: always ensure that parents are ordered before their children. TODO: is that even necessary?

/// A hierarchy of nodes.
pub struct Hierarchy<T>
where
    T: Sized,
{
    nodes: Vec<Node<T>>,
}

impl<T> Hierarchy<T>
where
    T: Sized,
{
    /// Initializes a new node based hierarchy.
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    /// Returns whether the given id is valid or not.
    fn is_valid(&self, id: Id) -> bool {
        id < self.nodes.len()
    }

    /// Adds a new node
    pub fn add_node(&mut self, item: T) -> Id {
        let id = self.nodes.len();

        self.nodes.push(Node::new(item));

        id
    }

    /// Adds a link to the parent node for a child.
    pub fn add_parent(&mut self, child: Id, parent: Id) {
        if self.is_valid(child) && self.is_valid(parent) {
            self.nodes[child].parent = Some(parent);
            self.nodes[parent].children.push(child);
        }
    }

    /// Returns a reference to the node.
    pub fn node(&self, id: Id) -> Option<&Node<T>> {
        if !self.is_valid(id) {
            return None;
        }

        Some(&self.nodes[id])
    }

    /// Returns a mutable reference to a Node
    pub fn node_mut(&mut self, id: Id) -> Option<&mut Node<T>> {
        if !self.is_valid(id) {
            return None;
        }

        Some(&mut self.nodes[id])
    }

    /// Returns a reference to a given item.
    pub fn item(&self, id: Id) -> Option<&T> {
        if !self.is_valid(id) {
            return None;
        }

        Some(&self.nodes[id].item)
    }

    /// Returns a mutable reference to a given item.
    pub fn item_mut(&mut self, id: Id) -> Option<&mut T> {
        if !self.is_valid(id) {
            return None;
        }

        Some(&mut self.nodes[id].item)
    }

    /// Returns a linear iterator for all items.
    pub fn iter_items<'a>(&'a self) -> Box<dyn Iterator<Item = &T> + 'a> {
        Box::new(self.nodes.iter().map(|t| &t.item))
    }

    /// Returns a mutable linear iterator for all items.
    pub fn iter_items_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut T> + 'a> {
        Box::new(self.nodes.iter_mut().map(|t| &mut t.item))
    }

    /// Returns a linear iterator for all nodes.
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (Id, &Node<T>)> + 'a> {
        Box::new(self.nodes.iter().enumerate())
    }

    /// Delete a given node and all it's children.
    pub fn delete(&mut self, id: Id) {
        if id > self.nodes.len() {
            return;
        }
        while let Some(child) = self.nodes[id].children.pop() {
            self.delete(child);
        }

        // If node is a child, remove it from the parent
        if let Some(parent) = self.nodes[id].parent {
            self.nodes[parent].remove_child(id);
        }

        // Remove node
        self.nodes.remove(id);

        // Reallocate children
        for node in self.nodes.iter_mut() {
            // Remove any children that reference deleted node
            node.remove_child(id);

            // Update parent
            if let Some(parent) = node.parent() {
                if parent > 0 && parent > id {
                    node.parent = Some(parent - 1);
                }
            }

            // Update children id offsets
            for child in node.children.iter_mut() {
                if *child > 0 && *child > id {
                    *child -= 1; //TODO: verify that this actually updated
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
