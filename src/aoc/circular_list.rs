use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use bare_metal_modulo::{MNum, ModNum};
use itertools::Itertools;

mod links {
    use super::*;

    pub struct CircularListNode<L: Links> {
        pub value: L::Node,
        pub links: L,
    }

    pub type NodeRefWeak<L> = Weak<RefCell<CircularListNode<L>>>;

    pub trait Links: Sized + Default {
        type Node;

        fn set(&mut self, previous: NodeRefWeak<Self>, next: NodeRefWeak<Self>);
        fn unlink_next(&mut self);
        fn link_after(&mut self, node: &NodeRefWeak<Self>);
        fn next(&self) -> Option<NodeRefWeak<Self>>;
        fn is_linked(&self) -> bool;
    }
}

use links::{CircularListNode, NodeRefWeak};

impl<A: links::Links> Links for A {}

pub trait Links: links::Links {}

pub struct SinglyLinked<T> {
    next: Option<NodeRefWeak<Self>>,
}
impl<T> Default for SinglyLinked<T> {
    fn default() -> Self {
        Self { next: None }
    }
}
impl<T> links::Links for SinglyLinked<T> {
    type Node = T;

    fn set(&mut self, _previous: NodeRefWeak<Self>, next: NodeRefWeak<Self>) {
        self.next = Some(next);
    }

    fn unlink_next(&mut self) {
        let next_rc = self.next.as_ref().unwrap().upgrade().unwrap();
        let next = &mut next_rc.deref().borrow_mut().links;
        self.next = next.next.take();
    }

    fn link_after(&mut self, node: &NodeRefWeak<Self>) {
        let next = self.next.take().unwrap();

        self.next = Some(node.clone());

        let node_rc = node.upgrade().unwrap();
        let node = &mut node_rc.deref().borrow_mut().links;

        node.next = Some(next);
    }

    fn next(&self) -> Option<NodeRefWeak<Self>> {
        self.next.clone()
    }

    fn is_linked(&self) -> bool {
        self.next.is_some()
    }
}

pub struct DoublyLinked<T> {
    previous: Option<NodeRefWeak<Self>>,
    next: Option<NodeRefWeak<Self>>,
}
impl<T> Default for DoublyLinked<T> {
    fn default() -> Self {
        Self {
            next: None,
            previous: None,
        }
    }
}
impl<T> links::Links for DoublyLinked<T> {
    type Node = T;

    fn set(&mut self, previous: NodeRefWeak<Self>, next: NodeRefWeak<Self>) {
        self.previous = Some(previous);
        self.next = Some(next);
    }

    fn unlink_next(&mut self) {
        let node_rc = self.next.as_ref().unwrap().upgrade().unwrap();
        let node_links = &mut node_rc.deref().borrow_mut().links;

        self.next = node_links.next.take();
        let this_node = node_links.previous.take().unwrap();

        let next_rc = self.next.as_ref().unwrap().upgrade().unwrap();
        let next_links = &mut next_rc.deref().borrow_mut().links;

        next_links.previous = Some(this_node);
    }

    fn link_after(&mut self, node: &NodeRefWeak<Self>) {
        let next_rc = self.next.as_ref().unwrap().upgrade().unwrap();
        let next_links = &mut next_rc.deref().borrow_mut().links;

        let this_node = next_links.previous.take().unwrap();

        let node_rc = node.upgrade().unwrap();
        let node_links = &mut node_rc.deref().borrow_mut().links;

        node_links.next = self.next.take();
        node_links.previous = Some(this_node);

        self.next = Some(node.clone());
        next_links.previous = Some(node.clone());
    }

    fn next(&self) -> Option<NodeRefWeak<Self>> {
        self.next.clone()
    }

    fn is_linked(&self) -> bool {
        self.previous.is_some() && self.next.is_some()
    }
}
impl<T> DoublyLinked<T> {
    fn previous(&self) -> Option<NodeRefWeak<Self>> {
        self.previous.clone()
    }
}

pub struct NodeRef<'a, L: Links> {
    list: &'a CircularList<L>,
    raw: Rc<RefCell<CircularListNode<L>>>,
}
impl<'a, L: Links> Clone for NodeRef<'a, L> {
    fn clone(&self) -> Self {
        Self {
            list: self.list,
            raw: self.raw.clone(),
        }
    }
}
impl<L: Links> PartialEq for NodeRef<'_, L> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.raw, &other.raw)
    }
}
impl<L: Links> Eq for NodeRef<'_, L> {}
impl<L: Links> std::fmt::Debug for NodeRef<'_, L>
where
    L::Node: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value())
    }
}
impl<'a, L: Links> NodeRef<'a, L> {
    pub fn value(&self) -> &'a L::Node {
        let p_val = self.with_ref(|n| &n.value as *const L::Node);
        unsafe { &*p_val }
    }
}
impl<'a, L: Links> NodeRef<'a, L> {
    fn from_weak(list: &'a CircularList<L>, weak: NodeRefWeak<L>) -> Self {
        Self {
            list,
            raw: weak.upgrade().unwrap(),
        }
    }

    fn with_ref<R>(&self, f: impl FnOnce(&CircularListNode<L>) -> R) -> R {
        f(self.raw.borrow().deref())
    }

    fn is_linked(&self) -> bool {
        self.with_ref(|n| n.links.is_linked())
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut CircularListNode<L>) -> R) -> R {
        f(self.raw.borrow_mut().deref_mut())
    }

    pub fn iter(&self, once: bool) -> CircularListNodeIterator<'a, L> {
        CircularListNodeIterator {
            list: self.list,
            next: Some(self.clone()),
            stop: once.then(|| self.clone()),
        }
    }

    pub fn next(&self) -> NodeRef<'a, L> {
        self.with_ref(|n| n.links.next().map(|r| NodeRef::from_weak(self.list, r)))
            .expect("cannot return the next node because this node is unlinked")
    }

    fn forward_index(&self, len_delta: isize, relative_index: isize) -> usize {
        ModNum::new(
            relative_index,
            isize::try_from(*self.list.len.borrow()).unwrap() + len_delta,
        )
        .a()
        .try_into()
        .unwrap()
    }

    pub fn remove_next(&mut self) -> NodeRef<'a, L> {
        assert!(
            self.is_linked(),
            "cannot remove next because this node is unlinked"
        );
        assert!(
            *self.list.len.borrow() > 1,
            "cannot remove because this is the last remaining linked node"
        );

        let node = self.next();
        self.with_mut(|n| n.links.unlink_next());

        *self.list.len.borrow_mut() -= 1;
        node
    }

    pub fn insert_after(&mut self, node: NodeRef<'a, L>) {
        assert!(
            std::ptr::eq(self.list, node.list),
            "cannot insert because the nodes are from different lists",
        );
        assert!(
            !node.is_linked(),
            "cannot insert the node because it is still linked",
        );
        assert!(
            self.is_linked(),
            "cannot insert because this node is not linked",
        );

        self.with_mut(|n| n.links.link_after(&Rc::downgrade(&node.raw)));
        *self.list.len.borrow_mut() += 1;
    }

    // If a single element is left has no effect
    pub fn shift_next(&mut self, relative_index: isize) {
        if *self.list.len.borrow() <= 1 {
            return;
        }

        let forward_index = self.forward_index(-1, relative_index);

        let mut insert_node = if forward_index > 0 {
            self.iter(false).nth(forward_index + 1).unwrap()
        } else {
            return;
        };

        let node = self.remove_next();
        insert_node.insert_after(node);
    }

    pub fn node_at(&self, relative_index: isize) -> NodeRef<'a, L> {
        let forward_index = self.forward_index(0, relative_index);

        self.iter(false).nth(forward_index).unwrap()
    }
}
impl<'a, T> NodeRef<'a, DoublyLinked<T>> {
    pub fn previous(&self) -> NodeRef<'a, DoublyLinked<T>> {
        self.with_ref(|n| n.links.previous().map(|r| NodeRef::from_weak(self.list, r)))
            .expect("cannot return the previous node because this node is unlinked")
    }

    pub fn shift(&mut self, relative_index: isize) {
        self.previous().shift_next(relative_index);
    }
}

pub struct CircularListNodeIterator<'a, L: Links> {
    list: &'a CircularList<L>,
    next: Option<NodeRef<'a, L>>,
    stop: Option<NodeRef<'a, L>>,
}
impl<L: Links> Clone for CircularListNodeIterator<'_, L> {
    fn clone(&self) -> Self {
        Self {
            list: self.list,
            next: self.next.clone(),
            stop: self.stop.clone(),
        }
    }
}
impl<L: Links> std::fmt::Debug for CircularListNodeIterator<'_, L>
where
    L::Node: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.clone();
        iter.stop = self.next.clone();

        write!(f, "[{}]", iter.map(|nr| format!("{nr:?}")).join(", "))
    }
}
impl<'a, L: Links> CircularListNodeIterator<'a, L> {
    fn fetch_next(
        &mut self,
        which: impl FnOnce(&CircularListNode<L>) -> Option<NodeRef<'a, L>>,
    ) -> Option<NodeRef<'a, L>> {
        let ret = self.next.clone();

        if let Some(current_node) = ret.as_ref() {
            self.next = current_node.with_ref(which).and_then(|next_node| {
                if let Some(stop_node) = self.stop.as_ref()
                    && *stop_node == next_node
                {
                    None
                } else {
                    Some(next_node)
                }
            })
        }

        ret
    }
}
impl<'a, L: Links> Iterator for CircularListNodeIterator<'a, L> {
    type Item = NodeRef<'a, L>;

    fn next(&mut self) -> Option<Self::Item> {
        self.fetch_next(|n| n.links.next().map(|r| NodeRef::from_weak(self.list, r)))
    }
}
impl<T> DoubleEndedIterator for CircularListNodeIterator<'_, DoublyLinked<T>> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.fetch_next(|n| {
            n.links
                .previous
                .as_ref()
                .map(|r| NodeRef::from_weak(self.list, r.clone()))
        })
    }
}

pub struct CircularList<L: Links> {
    nodes: Vec<Rc<RefCell<CircularListNode<L>>>>,
    len: RefCell<usize>,
}
impl<L: Links> std::fmt::Debug for CircularList<L>
where
    L::Node: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.iter_const().next().unwrap())
    }
}
impl<L: Links> CircularList<L> {
    pub fn new(iter: impl Iterator<Item = L::Node>) -> Option<Self> {
        // Create initial list of nodes
        let nodes = iter
            .map(|value| {
                Rc::new(RefCell::new(CircularListNode {
                    value,
                    links: L::default(),
                }))
            })
            .collect_vec();

        assert!(
            !nodes.is_empty(),
            "cannot create a circular list with no elements"
        );

        // Now add linked list references
        let len = nodes.len();
        for (idx, node) in nodes.iter().enumerate() {
            let idx = ModNum::new(idx, len);

            let mut node = node.as_ref().borrow_mut();
            node.links.set(
                Rc::downgrade(&nodes[(idx - 1).a()]),
                Rc::downgrade(&nodes[(idx + 1).a()]),
            )
        }

        (!nodes.is_empty()).then_some(Self {
            nodes,
            len: RefCell::new(len),
        })
    }

    pub fn original_len(&self) -> usize {
        self.nodes.len()
    }

    pub fn len(&self) -> usize {
        *self.len.borrow()
    }

    pub fn iter_const(&self) -> impl Iterator<Item = NodeRef<L>> + '_ {
        self.nodes.iter().map(|n| NodeRef {
            list: self,
            raw: n.clone(),
        })
    }
}
