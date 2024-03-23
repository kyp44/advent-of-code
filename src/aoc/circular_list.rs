use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use bare_metal_modulo::{MNum, ModNum};
use itertools::Itertools;

struct CircularListNode<T> {
    value: T,
    previous: Option<NodeRefRaw<T>>,
    next: Option<NodeRefRaw<T>>,
}
impl<T> CircularListNode<T> {
    pub fn previous<'a>(&self, list: &'a CircularList<T>) -> Option<NodeRef<'a, T>> {
        self.previous.as_ref().map(|nr| NodeRef {
            list,
            raw: nr.clone(),
        })
    }

    pub fn next<'a>(&self, list: &'a CircularList<T>) -> Option<NodeRef<'a, T>> {
        self.next.as_ref().map(|nr| NodeRef {
            list,
            raw: nr.clone(),
        })
    }
}

type NodeRefRaw<T> = Weak<RefCell<CircularListNode<T>>>;

pub struct NodeRef<'a, T> {
    list: &'a CircularList<T>,
    raw: NodeRefRaw<T>,
}
impl<'a, T> Clone for NodeRef<'a, T> {
    fn clone(&self) -> Self {
        Self {
            list: self.list,
            raw: self.raw.clone(),
        }
    }
}
impl<T> PartialEq for NodeRef<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw.ptr_eq(&other.raw)
    }
}
impl<T> Eq for NodeRef<'_, T> {}
impl<T: std::fmt::Debug> std::fmt::Debug for NodeRef<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value())
    }
}
impl<'a, T> NodeRef<'a, T> {
    pub fn value(&self) -> &'a T {
        let p_val = self.with_ref(|n| &n.value as *const T);
        unsafe { &*p_val }
    }
}
impl<'a, T> NodeRef<'a, T> {
    fn with_ref<R>(&self, f: impl FnOnce(&CircularListNode<T>) -> R) -> R {
        f(self.raw.upgrade().unwrap().as_ref().borrow().deref())
    }

    fn is_linked(&self) -> bool {
        self.with_ref(|n| n.previous.is_some() && n.next.is_some())
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut CircularListNode<T>) -> R) -> R {
        f(self
            .raw
            .upgrade()
            .unwrap()
            .as_ref()
            .borrow_mut()
            .deref_mut())
    }

    pub fn previous(&self) -> Option<NodeRef<'a, T>> {
        self.with_ref(|n| n.previous(self.list))
    }

    pub fn next(&self) -> Option<NodeRef<'a, T>> {
        self.with_ref(|n| n.next(self.list))
    }

    pub fn iter(&self, once: bool) -> CircularListNodeIterator<'a, T> {
        CircularListNodeIterator {
            list: self.list,
            next: Some(self.clone()),
            stop: once.then(|| self.clone()),
        }
    }

    pub fn remove(&mut self) {
        assert!(
            self.is_linked(),
            "cannot remove because the node is already unlinked"
        );
        assert!(
            *self.list.len.borrow() > 1,
            "cannot remove because this is the last remaining linked node"
        );

        self.with_mut(|node| {
            let node_next = node.next(self.list).unwrap();

            node.previous(self.list)
                .unwrap()
                .with_mut(|p| p.next = node.next.take());

            node_next.with_mut(|n| n.previous = node.previous.take());
            *self.list.len.borrow_mut() -= 1;
        })
    }

    pub fn insert_after(&mut self, node: NodeRef<'a, T>) {
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

        let next = self.with_ref(|n| n.next(self.list)).unwrap();

        node.with_mut(|n| {
            n.previous = Some(self.raw.clone());
            n.next = Some(next.raw.clone());
        });

        self.with_mut(|n| n.next = Some(node.raw.clone()));
        next.with_mut(|n| n.previous = Some(node.raw.clone()));
        *self.list.len.borrow_mut() += 1;
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

    // If a single element is left has no effect
    pub fn shift(&mut self, relative_index: isize) {
        if *self.list.len.borrow() <= 1 {
            return;
        }

        let forward_index = self.forward_index(-1, relative_index);

        let mut insert_node = if forward_index > 0 {
            self.iter(false).nth(forward_index).unwrap()
        } else {
            return;
        };

        self.remove();
        insert_node.insert_after(self.clone());
    }

    pub fn node_at(&self, relative_index: isize) -> NodeRef<'a, T> {
        let forward_index = self.forward_index(0, relative_index);

        self.iter(false).nth(forward_index).unwrap()
    }
}

#[derive(Clone)]
pub struct CircularListNodeIterator<'a, T> {
    list: &'a CircularList<T>,
    next: Option<NodeRef<'a, T>>,
    stop: Option<NodeRef<'a, T>>,
}
impl<T: std::fmt::Debug + Clone> std::fmt::Debug for CircularListNodeIterator<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.clone();
        iter.stop = self.next.clone();

        write!(f, "[{}]", iter.map(|nr| format!("{nr:?}")).join(", "))
    }
}
impl<'a, T> CircularListNodeIterator<'a, T> {
    fn fetch_next(
        &mut self,
        which: impl FnOnce(&CircularListNode<T>) -> Option<NodeRef<'a, T>>,
    ) -> Option<NodeRef<'a, T>> {
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
impl<'a, T> Iterator for CircularListNodeIterator<'a, T> {
    type Item = NodeRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.fetch_next(|n| n.next(self.list))
    }
}
impl<T> DoubleEndedIterator for CircularListNodeIterator<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.fetch_next(|n| n.previous(self.list))
    }
}

pub struct CircularList<T> {
    nodes: Vec<Rc<RefCell<CircularListNode<T>>>>,
    len: RefCell<usize>,
}
impl<T: std::fmt::Debug> std::fmt::Debug for CircularList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.iter_const().next().unwrap())
    }
}
impl<T> CircularList<T> {
    pub fn new(iter: impl Iterator<Item = T>) -> Option<Self> {
        // Create initial list of nodes
        let nodes = iter
            .map(|value| {
                Rc::new(RefCell::new(CircularListNode {
                    value,
                    previous: None,
                    next: None,
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
            node.previous = Some(Rc::downgrade(&nodes[(idx - 1).a()]));
            node.next = Some(Rc::downgrade(&nodes[(idx + 1).a()]));
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

    pub fn iter_const(&self) -> impl Iterator<Item = NodeRef<T>> + '_ {
        self.nodes.iter().map(|n| NodeRef {
            list: self,
            raw: Rc::downgrade(n),
        })
    }
}
