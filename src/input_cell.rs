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

    pub fn parent_iter(&self) -> Map<slice::Iter<Weak<RefCell<Node<'a, T>>>>, fn(&Weak<RefCell<Node<'a, T>>>) -> Rc<RefCell<Node<'a, T>>>> {
        self.parents.iter().map(|e| e.upgrade().unwrap())
    }
}

/*impl<'a, T: Copy + Debug + PartialEq> IntoIterator for &'a InputCellType<T> {
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
}*/
