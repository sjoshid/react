use crate::{CallbackId, Node, RemoveCallbackError};
use std::cell::RefCell;
use std::fmt::Debug;
use std::iter::Map;
use std::rc::{Rc, Weak};
use std::slice;

pub struct ComputeCellType<'a, T> {
    parents: Vec<Weak<RefCell<Node<'a, T>>>>,
    children: Vec<Rc<RefCell<Node<'a, T>>>>,
    compute_function: Rc<dyn Fn(&[T]) -> T>,
    callback_function: Vec<Option<Box<dyn FnMut(T) + 'a>>>,
    callback_index: usize,
}

impl<'a, T: Copy + Debug + PartialEq> ComputeCellType<'a, T> {
    pub fn new<F: 'static + Fn(&[T]) -> T>(
        children: Vec<Rc<RefCell<Node<'a, T>>>>,
        compute_function: F,
    ) -> Self {
        Self {
            parents: vec![],
            children,
            compute_function: Rc::new(compute_function),
            callback_function: vec![],
            callback_index: 0,
        }
    }

    pub fn add_parent(&mut self, node: Rc<RefCell<Node<'a, T>>>) {
        let down = Rc::downgrade(&node);
        self.parents.push(down);
    }

    pub fn rerun_compute_function(&self, values: &[T]) -> T {
        (self.compute_function)(values)
    }

    pub fn add_callback<F: 'a + FnMut(T)>(&mut self, callback: F) -> Option<CallbackId> {
        let len = self.callback_function.len();
        self.callback_function.push(Some(Box::new(callback)));
        let id = CallbackId::new(len);
        Some(id)
    }

    pub fn invoke_callback(&mut self, value: T) {
        self.callback_function
            .iter_mut()
            .filter(|e| e.is_some())
            .for_each(|e| {
                (e.as_mut().unwrap())(value);
            });
    }

    pub fn remove_callback(&mut self, callback_id: CallbackId) -> Result<(), RemoveCallbackError> {
        let index = callback_id.id;

        match self.callback_function.get(index) {
            None => Err(RemoveCallbackError::NonexistentCallback),
            Some(cb) if cb.is_some() => {
                std::mem::replace(&mut self.callback_function[index], None);
                Ok(())
            }
            _ => Err(RemoveCallbackError::NonexistentCallback),
        }
    }

    pub fn parent_iter(
        &self,
    ) -> Map<
        slice::Iter<Weak<RefCell<Node<'a, T>>>>,
        fn(&Weak<RefCell<Node<'a, T>>>) -> Rc<RefCell<Node<'a, T>>>,
    > {
        self.parents.iter().map(|e| e.upgrade().unwrap())
    }

    pub fn children_iter(&self) -> slice::Iter<Rc<RefCell<Node<'a, T>>>> {
        self.children.iter()
    }
}
