//use std::borrow::BorrowMut;
use crate::Node;
use std::cell::{RefCell, RefMut};
use std::fmt::Debug;
use std::iter::Map;
use std::rc::{Rc, Weak};
use std::slice;
use std::slice::{Iter, IterMut};

pub struct InputCellType<'a, T> {
    parents: Vec<Weak<RefCell<Node<'a, T>>>>,
}

impl<'a, T: Copy + Debug + PartialEq> InputCellType<'a, T> {
    pub fn new(parents: Vec<Weak<RefCell<Node<'a, T>>>>) -> Self {
        Self { parents }
    }

    pub fn add_parent(&mut self, node: Rc<RefCell<Node<'a, T>>>) {
        let down = Rc::downgrade(&node);
        self.parents.push(down);
    }

    pub fn parent_iter(
        &self,
    ) -> Map<
        slice::Iter<Weak<RefCell<Node<'a, T>>>>,
        fn(&Weak<RefCell<Node<'a, T>>>) -> Rc<RefCell<Node<'a, T>>>,
    > {
        self.parents.iter().map(|e| e.upgrade().unwrap())
    }
}
