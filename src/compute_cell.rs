use crate::Node;
use std::cell::RefCell;
use std::fmt::Debug;
use std::iter::Map;
use std::rc::{Rc, Weak};
use std::slice;

pub struct ComputeCellType<T> {
    parents: Vec<Weak<RefCell<Node<T>>>>,
    children: Vec<Rc<RefCell<Node<T>>>>,
    compute_function: Rc<dyn Fn(&[T]) -> T>,
    callback_function: Option<Box<dyn FnMut(T)>>,
}

impl<T: Copy + Debug + PartialEq> ComputeCellType<T> {
    pub fn new<F: 'static + Fn(&[T]) -> T>(
        children: Vec<Rc<RefCell<Node<T>>>>,
        compute_function: F,
    ) -> Self {
        Self {
            parents: vec![],
            children,
            compute_function: Rc::new(compute_function),
            callback_function: None,
        }
    }

    pub fn add_parent(&mut self, node: Rc<RefCell<Node<T>>>) {
        let down = Rc::downgrade(&node);
        self.parents.push(down);
    }

    pub fn rerun_compute_function(&self, values: &[T]) -> T {
        (self.compute_function)(values)
    }

    pub fn add_callback<F: 'static + FnMut(T)>(&mut self, callback: F) {
        self.callback_function = Some(Box::new(callback));
    }

    pub fn invoke_callback(&mut self, value: T) {
        if let Some(cb) = &mut self.callback_function {
            cb(value);
        }
    }

    pub fn remove_callback(&mut self) {
        self.callback_function = None;
    }
}

impl<'a, T: Copy + Debug + PartialEq> IntoIterator for &'a mut ComputeCellType<T> {
    type Item = Rc<RefCell<Node<T>>>;
    type IntoIter = Map<
        slice::IterMut<'a, Weak<RefCell<Node<T>>>>,
        fn(&'a mut Weak<RefCell<Node<T>>>) -> Rc<RefCell<Node<T>>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.parents.iter_mut().map(|e| e.upgrade().unwrap())
    }
}

impl<'a, T: Copy + Debug + PartialEq> IntoIterator for &'a ComputeCellType<T> {
    type Item = &'a Rc<RefCell<Node<T>>>;
    type IntoIter = slice::Iter<'a, Rc<RefCell<Node<T>>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.iter()
    }
}
