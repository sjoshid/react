//use std::borrow::BorrowMut;
use crate::Node;
use std::cell::{RefCell, RefMut};
use std::fmt::Debug;
use std::iter::Map;
use std::rc::{Rc, Weak};
use std::slice;
use std::slice::{Iter, IterMut};

pub struct InputCellType<T> {
    parents: Vec<Weak<RefCell<Node<T>>>>,
}

impl<T: Copy + Debug + PartialEq> InputCellType<T> {
    pub fn new(parents: Vec<Weak<RefCell<Node<T>>>>) -> Self {
        Self { parents }
    }

    pub fn add_parent(&mut self, node: Rc<RefCell<Node<T>>>) {
        let down = Rc::downgrade(&node);
        self.parents.push(down);
    }
}

impl<'a, T: Copy + Debug + PartialEq> IntoIterator for &'a InputCellType<T> {
    type Item = &'a Weak<RefCell<Node<T>>>;
    type IntoIter = slice::Iter<'a, Weak<RefCell<Node<T>>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.parents.iter()
    }
}

impl<'a, T: Copy + Debug + PartialEq> IntoIterator for &'a mut InputCellType<T> {
    type Item = Rc<RefCell<Node<T>>>;
    type IntoIter = Map<
        slice::IterMut<'a, Weak<RefCell<Node<T>>>>,
        fn(&'a mut Weak<RefCell<Node<T>>>) -> Rc<RefCell<Node<T>>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.parents.iter_mut().map(|e| e.upgrade().unwrap())
    }
}
