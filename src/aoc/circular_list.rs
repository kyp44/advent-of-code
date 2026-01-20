//! For singly and doubly linked circular lists.
//!
//! The main item is the [`CircularList`].
use std::{
    cell::RefCell,
    iter::FusedIterator,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use bare_metal_modulo::{MNum, ModNum};
use itertools::Itertools;

/// Private module containing the actual [`Links`](links::Links) trait.
mod links {
    use super::*;

    /// The main node containing the data and links.
    pub struct CircularListNode<L: Links> {
        /// The data value.
        pub value: L::NodeData,
        /// The links to adjacent node or nodes.
        pub links: L,
    }

    /// A weak reference to an adjacent node.
    pub type NodeRefWeak<L> = Weak<RefCell<CircularListNode<L>>>;

    /// The private trait implemented by structures containing the node links.
    pub trait Links: Sized + Default {
        /// The node data type.
        type NodeData;

        /// Sets the links given the previous and next node references.
        fn set(&mut self, previous: NodeRefWeak<Self>, next: NodeRefWeak<Self>);
        /// Unlinks the next node in the list.
        fn unlink_next(&mut self);
        /// Inserts a node into the list after this one.
        fn link_after(&mut self, node: &NodeRefWeak<Self>);
        /// Returns a reference to the next node if this one is linked.
        fn next(&self) -> Option<NodeRefWeak<Self>>;
        /// Returns whether this node is linked.
        fn is_linked(&self) -> bool;
    }
}

use links::{CircularListNode, NodeRefWeak};

/// Implementors of this trait represent different ways in which the nodes
/// of a circular list can be linked together.
pub trait Links: links::Links {}

/// Links for a singly linked list.
///
/// The type parameter `T` is the type of the node data.
pub struct SinglyLinked<T> {
    /// The reference to the next node, if this node is linked.
    next: Option<NodeRefWeak<Self>>,
}
impl<T> Default for SinglyLinked<T> {
    fn default() -> Self {
        Self { next: None }
    }
}
impl<T> links::Links for SinglyLinked<T> {
    type NodeData = T;

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
impl<T> Links for SinglyLinked<T> {}

/// Links for a a doubly linked list.
///
/// The type parameter `T` is the type of the node data.
pub struct DoublyLinked<T> {
    /// The reference to the previous node, if this node is linked.
    previous: Option<NodeRefWeak<Self>>,
    /// The reference to the next node, if this node is linked.
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
    type NodeData = T;

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
impl<T> Links for DoublyLinked<T> {}
impl<T> DoublyLinked<T> {
    /// Returns a reference to the previous node, if this node is linked.
    fn previous(&self) -> Option<NodeRefWeak<Self>> {
        self.previous.clone()
    }
}

/// A reference to a node in a particular [`CircularList`].
///
/// The node may be linked or unlinked, that is part of the circular
/// list or removed from it.
/// Node references can initially be obtained using
/// [`CircularList::iter_const`].
pub struct NodeRef<'a, L: Links> {
    /// The list to which the node belongs, even if unlinked.
    list: &'a CircularList<L>,
    /// The strong reference to the [`CircularListNode`].
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
    L::NodeData: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value())
    }
}
impl<'a, L: Links> NodeRef<'a, L> {
    /// Returns a reference to the node data.
    pub fn value(&self) -> &'a L::NodeData {
        let p_val = self.with_ref(|n| &n.value as *const L::NodeData);
        unsafe { &*p_val }
    }
}
impl<'a, L: Links> NodeRef<'a, L> {
    /// Creates a reference given the raw weak reference and the list to
    /// which the node belongs.
    ///
    /// This will panic if the weak reference cannot be upgraded.
    fn from_weak(list: &'a CircularList<L>, weak: NodeRefWeak<L>) -> Self {
        Self {
            list,
            raw: weak.upgrade().unwrap(),
        }
    }

    /// Runs a closure to which a direct reference to the [`CircularListNode`]
    /// is passed.
    fn with_ref<R>(&self, f: impl FnOnce(&CircularListNode<L>) -> R) -> R {
        f(self.raw.borrow().deref())
    }

    /// Runs a closure to which a direct, mutable reference to the [`CircularListNode`]
    /// is passed.
    fn with_mut<R>(&self, f: impl FnOnce(&mut CircularListNode<L>) -> R) -> R {
        f(self.raw.borrow_mut().deref_mut())
    }

    /// Returns whether this node is linked or not.
    fn is_linked(&self) -> bool {
        self.with_ref(|n| n.links.is_linked())
    }

    /// Returns an forward [`Iterator`] over the circular list of linked nodes.
    ///
    /// The the iterator may go around the circle `once`, or an infinite number
    /// of times.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use aoc::circular_list::{CircularList, DoublyLinked, SinglyLinked};
    /// use itertools::Itertools;
    ///
    /// let list: CircularList<SinglyLinked<_>> = CircularList::new([1, 2, 3, 4, 5, 6]);
    /// let third_node = list.iter_const().nth(2).unwrap();
    ///
    /// assert_eq!(
    ///     third_node.iter(true).map(|n| *n.value()).collect_vec(),
    ///     vec![3, 4, 5, 6, 1, 2]
    /// );
    /// assert_eq!(
    ///     third_node
    ///         .iter(false)
    ///         .map(|n| *n.value())
    ///         .take(15)
    ///         .collect_vec(),
    ///     vec![3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5]
    /// );
    ///
    /// let list: CircularList<DoublyLinked<_>> = CircularList::new([1, 2, 3, 4, 5, 6]);
    /// let third_node = list.iter_const().nth(2).unwrap();
    ///
    /// assert_eq!(
    ///     third_node
    ///         .iter(true)
    ///         .rev()
    ///         .map(|n| *n.value())
    ///         .collect_vec(),
    ///     vec![3, 2, 1, 6, 5, 4]
    /// );
    /// assert_eq!(
    ///     third_node
    ///         .iter(false)
    ///         .rev()
    ///         .map(|n| *n.value())
    ///         .take(15)
    ///         .collect_vec(),
    ///     vec![3, 2, 1, 6, 5, 4, 3, 2, 1, 6, 5, 4, 3, 2, 1]
    /// );
    /// ```
    pub fn iter(&self, once: bool) -> CircularListNodeIterator<'a, L> {
        CircularListNodeIterator {
            list: self.list,
            next: Some(self.clone()),
            stop: once.then(|| self.clone()),
        }
    }

    /// Returns a reference to the next node, panicking if this node is unlinked.
    pub fn next(&self) -> NodeRef<'a, L> {
        self.with_ref(|n| n.links.next().map(|r| NodeRef::from_weak(self.list, r)))
            .expect("cannot return the next node because this node is unlinked")
    }

    /// Returns an unsigned index in the forward direction to get to a node
    /// from this one, given a signed index relative to this node.
    ///
    /// The unsigned index is guaranteed to be less than the length of the list.
    fn forward_index(&self, len_delta: isize, relative_index: isize) -> usize {
        ModNum::new(
            relative_index,
            isize::try_from(*self.list.len.borrow()).unwrap() + len_delta,
        )
        .a()
        .try_into()
        .unwrap()
    }

    /// Removes (unlinks) the next node from the circular list and returns a
    /// reference to it.
    ///
    /// Note that this does not drop the node, as it is still owned by the
    /// [`CircularList`].
    /// If the reference is discarded, it can be retrieved again using
    /// [`CircularList::iter_const`].
    ///
    /// # Panics
    /// This will panic if this node is unlinked or it is the sole remaining node.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use aoc::circular_list::{CircularList, SinglyLinked};
    /// use itertools::Itertools;
    ///
    /// let list: CircularList<SinglyLinked<_>> = CircularList::new([1, 2, 3, 4, 5, 6]);
    /// let mut third_node = list.iter_const().nth(2).unwrap();
    ///
    /// third_node.remove_next();
    /// assert_eq!(
    ///     third_node.iter(true).map(|n| *n.value()).collect_vec(),
    ///     vec![3, 5, 6, 1, 2]
    /// );
    /// ```
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

    /// Inserts a node after this one in the list.
    ///
    /// # Panics
    /// This will panic if the nodes are from different [`CircularList`] objects,
    /// if the insertion node is still linked, or if this node is unlinked.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use aoc::circular_list::{CircularList, SinglyLinked};
    /// use itertools::Itertools;
    ///
    /// let list: CircularList<SinglyLinked<_>> = CircularList::new([1, 2, 3, 4, 5, 6]);
    /// let mut first_node = list.iter_const().next().unwrap();
    /// let mut fourth_node = list.iter_const().nth(3).unwrap();
    ///
    /// let fifth_node = fourth_node.remove_next();
    /// first_node.insert_after(fifth_node);
    /// assert_eq!(
    ///     first_node.iter(true).map(|n| *n.value()).collect_vec(),
    ///     vec![1, 5, 2, 3, 4, 6]
    /// );
    /// ```
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

    /// Shifts the *next* node forward or backward in the list.
    ///
    /// The `relative_index` is how many positions to shift the next node, with
    /// a negative index shifting backward and positive index shifting forward.
    /// Note that this is an efficient operation even for very large values of
    /// `relative_index` in that this is converted to an index modulo the list
    /// size prior to following the links.
    ///
    /// # Panics
    /// This will panic if this node is unlinked or it is the sole remaining node.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use aoc::circular_list::{CircularList, SinglyLinked};
    /// use itertools::Itertools;
    ///
    /// let list: CircularList<SinglyLinked<_>> = CircularList::new([1, 2, 3, 4, 5, 6]);
    /// let first_node = list.iter_const().next().unwrap();
    /// let mut fourth_node = list.iter_const().nth(3).unwrap();
    ///
    /// fourth_node.shift_next(-13);
    /// assert_eq!(
    ///     first_node.iter(true).map(|n| *n.value()).collect_vec(),
    ///     vec![1, 5, 2, 3, 4, 6]
    /// );
    ///
    /// fourth_node.shift_next(52);
    /// assert_eq!(
    ///     first_node.iter(true).map(|n| *n.value()).collect_vec(),
    ///     vec![1, 5, 6, 2, 3, 4]
    /// );
    /// ```
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

    /// Returns a reference to the node at the specified index, relative to
    /// this node.
    ///
    /// A negative `relative_index` selects nodes behind this one.
    /// Note that this is an efficient operation even for very large values of
    /// `relative_index` in that this is converted to an index modulo the list
    /// size prior to following the links.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use aoc::circular_list::{CircularList, SinglyLinked};
    ///
    /// let list: CircularList<SinglyLinked<_>> = CircularList::new([1, 2, 3, 4, 5, 6]);
    /// let first_node = list.iter_const().next().unwrap();
    ///
    /// assert_eq!(*first_node.node_at(0).value(), 1);
    /// assert_eq!(*first_node.node_at(-2).value(), 5);
    /// assert_eq!(*first_node.node_at(-11).value(), 2);
    /// assert_eq!(*first_node.node_at(4).value(), 5);
    /// assert_eq!(*first_node.node_at(13).value(), 2);
    /// ```
    pub fn node_at(&self, relative_index: isize) -> NodeRef<'a, L> {
        let forward_index = self.forward_index(0, relative_index);

        self.iter(false).nth(forward_index).unwrap()
    }
}
impl<'a, T> NodeRef<'a, DoublyLinked<T>> {
    /// Returns a reference to the previous node, panicking if this node is unlinked.
    pub fn previous(&self) -> NodeRef<'a, DoublyLinked<T>> {
        self.with_ref(|n| n.links.previous().map(|r| NodeRef::from_weak(self.list, r)))
            .expect("cannot return the previous node because this node is unlinked")
    }

    /// Shifts *this* node forward or backward in the list.
    ///
    /// The `relative_index` is how many positions to shift the next node, with
    /// a negative index shifting backward and positive index shifting forward.
    /// Note that this is an efficient operation even for very large values of
    /// `relative_index` in that this is converted to an index modulo the list
    /// size prior to following the links.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use aoc::circular_list::{CircularList, DoublyLinked};
    /// use itertools::Itertools;
    ///
    /// let list: CircularList<DoublyLinked<_>> = CircularList::new([1, 2, 3, 4, 5, 6]);
    /// let first_node = list.iter_const().next().unwrap();
    /// let mut fourth_node = list.iter_const().nth(3).unwrap();
    ///
    /// fourth_node.shift(-13);
    /// assert_eq!(
    ///     first_node.iter(true).map(|n| *n.value()).collect_vec(),
    ///     vec![1, 2, 3, 5, 6, 4]
    /// );
    ///
    /// fourth_node.shift(52);
    /// assert_eq!(
    ///     first_node.iter(true).map(|n| *n.value()).collect_vec(),
    ///     vec![1, 2, 4, 3, 5, 6]
    /// );
    /// ```
    pub fn shift(&mut self, relative_index: isize) {
        self.previous().shift_next(relative_index);
    }
}

/// [`Iterator`] over the nodes in a circular list.
///
/// Refer to [`NodeRef::iter`] for more details.
pub struct CircularListNodeIterator<'a, L: Links> {
    /// The list to which all nodes belong.
    list: &'a CircularList<L>,
    /// The next node to be returned, or [`None`] if the iteration is complete.
    next: Option<NodeRef<'a, L>>,
    /// The node at which to stop iteration, or [`None`] if the we are iterating
    /// around the circle an infinite number of times.
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
    L::NodeData: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.clone();
        iter.stop.clone_from(&self.next);

        write!(f, "[{}]", iter.map(|nr| format!("{nr:?}")).join(", "))
    }
}
impl<'a, L: Links> CircularListNodeIterator<'a, L> {
    /// Returns the next node, if any, given a closure `which` that selects the next
    /// node from the the [`Link`] implementor.
    fn fetch_next(
        &mut self,
        which: impl FnOnce(&L) -> Option<NodeRefWeak<L>>,
    ) -> Option<NodeRef<'a, L>> {
        let ret = self.next.clone();

        if let Some(current_node) = ret.as_ref() {
            self.next = current_node
                .with_ref(|n| which(&n.links))
                .and_then(|next_weak| {
                    let next_node = NodeRef::from_weak(self.list, next_weak);

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
        self.fetch_next(|links| links.next())
    }
}
impl<L: Links> FusedIterator for CircularListNodeIterator<'_, L> {}
impl<T> DoubleEndedIterator for CircularListNodeIterator<'_, DoublyLinked<T>> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.fetch_next(|links| links.previous())
    }
}

/// A circular linked list.
///
/// The type parameter `L` determines whether the list is [`SinglyLinked`] or
/// [`DoublyLinked`], the latter of which enables backward iteration through
/// the list.
/// Many operations to mutate the circular list are functions of [`NodeRef`]
/// rather than functions here.
pub struct CircularList<L: Links> {
    /// The constant set of owned nodes.
    ///
    /// Nodes are never removed from this even if they are unlinked from
    /// the circular list.
    nodes: Vec<Rc<RefCell<CircularListNode<L>>>>,
    /// The current length of the circular list, noting that unlinking a node
    /// will decrement this while not affecting the constant set of nodes.
    len: RefCell<usize>,
}
impl<L: Links> std::fmt::Debug for CircularList<L>
where
    L::NodeData: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.iter_const().next().unwrap())
    }
}
impl<L: Links> CircularList<L> {
    /// Creates a new circular list from the items yielded from `iter`.
    ///
    /// The initial list is in the same order as `iter` but is of course circular
    /// with the first and last items joined.
    ///
    /// # Panics
    /// This will panic if `iter` is empty as the circular list must always have
    /// at least one element.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use aoc::circular_list::{CircularList, SinglyLinked};
    /// use itertools::Itertools;
    ///
    /// let list: CircularList<SinglyLinked<_>> =
    ///     CircularList::new([1, 2, 3, 4, 5].into_iter().map(|x| -2 * x));
    ///
    /// assert_eq!(
    ///     list.iter_const().map(|n| *n.value()).collect_vec(),
    ///     vec![-2, -4, -6, -8, -10]
    /// );
    /// ```
    pub fn new(iter: impl IntoIterator<Item = L::NodeData>) -> Self {
        // Create initial list of nodes
        let nodes = iter
            .into_iter()
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

        Self {
            nodes,
            len: RefCell::new(len),
        }
    }

    /// Returns the length of the original circular list, that is the size of the `iter` passed
    /// to [`new`](CircularList::new).
    ///
    /// Once the list is created, this will forever be constant.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use aoc::circular_list::{CircularList, SinglyLinked};
    ///
    /// let list: CircularList<SinglyLinked<_>> = CircularList::new([1, 2, 3, 4, 5, 6]);
    /// let mut first_node = list.iter_const().next().unwrap();
    ///
    /// assert_eq!(list.original_len(), 6);
    ///
    /// first_node.remove_next();
    /// first_node.remove_next();
    /// assert_eq!(list.original_len(), 6);
    /// ```
    pub fn original_len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the current length of the list, which will be less than or equal to
    /// [`original_len`](CircularList::original_len), and will always be at least one.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use aoc::circular_list::{CircularList, SinglyLinked};
    ///
    /// let list: CircularList<SinglyLinked<_>> = CircularList::new([1, 2, 3, 4, 5, 6]);
    /// let mut first_node = list.iter_const().next().unwrap();
    ///
    /// assert_eq!(list.len(), 6);
    ///
    /// first_node.remove_next();
    /// first_node.remove_next();
    /// assert_eq!(list.len(), 4);
    /// ```
    pub fn len(&self) -> usize {
        *self.len.borrow()
    }

    /// Returns whether the list is empty.
    ///
    /// This should always be `false` since a list must always contain
    /// at least one element.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an [`Iterator`] of node references over the *original*, constant
    /// list.
    ///
    /// This can always be used to retrieve nodes that were removed from the list
    /// and then discarded.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use aoc::prelude::*;
    /// use aoc::circular_list::{CircularList, SinglyLinked};
    /// use itertools::Itertools;
    ///
    /// let list: CircularList<SinglyLinked<_>> = CircularList::new([1, 2, 3, 4, 5, 6]);
    /// let mut first_node = list.iter_const().next().unwrap();
    ///
    /// assert_eq!(
    ///     list.iter_const().map(|n| *n.value()).collect_vec(),
    ///     vec![1, 2, 3, 4, 5, 6]
    /// );
    ///
    /// first_node.remove_next();
    /// first_node.remove_next();
    /// assert_eq!(
    ///     list.iter_const().map(|n| *n.value()).collect_vec(),
    ///     vec![1, 2, 3, 4, 5, 6]
    /// );
    /// ```
    pub fn iter_const(&self) -> impl Iterator<Item = NodeRef<'_, L>> + '_ {
        self.nodes.iter().map(|n| NodeRef {
            list: self,
            raw: n.clone(),
        })
    }
}
