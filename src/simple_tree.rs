use crate::{CallbackId, CellId, ComputeCellType, InputCellType, RemoveCallbackError};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

enum Type<'a, T> {
    IC(InputCellType<'a, T>),
    CC(ComputeCellType<'a, T>),
}

pub struct Node<'a, T> {
    node_value: T,
    t: Type<'a, T>,
}

impl<'a, T: Copy + Debug + PartialEq> Node<'a, T> {
    pub fn create_input(value: T) -> Self {
        Self {
            node_value: value,
            t: Type::IC(InputCellType::new(vec![])),
        }
    }

    pub fn create_compute<F: 'static + Fn(&[T]) -> T>(
        children: Vec<Rc<RefCell<Node<'a, T>>>>,
        compute_func: F,
    ) -> Result<Rc<RefCell<Node<'a, T>>>, CellId> {
        let vals: Vec<T> = children
            .iter()
            .map(|v| v.as_ref().borrow().node_value)
            .collect();
        let vals = vals.as_slice();
        let value = compute_func(vals);

        let c1 = children.clone();
        let cct = ComputeCellType::new(children, compute_func);

        let ccn = Self {
            node_value: value,
            t: Type::CC(cct),
        };
        let rc = Rc::new(RefCell::new(ccn));

        c1.iter().for_each(|c| match &mut c.borrow_mut().t {
            Type::IC(ict) => {
                ict.add_parent(rc.clone());
            }
            Type::CC(cct) => {
                cct.add_parent(rc.clone());
            }
        });

        Ok(rc)
    }

    pub fn set_value(&mut self, value: T) {
        self.node_value = value;
        match &mut self.t {
            Type::IC(ict) => {
                for parent in ict.parent_iter() {
                    parent.borrow_mut().calculate_new_value(value);
                }
            }
            Type::CC(cct) => {
                for parent in cct.parent_iter() {
                    parent.borrow_mut().calculate_new_value(value);
                }
            }
        }
    }

    pub fn get_value(&self) -> T {
        self.node_value
    }

    pub fn add_callback<F: 'a + FnMut(T)>(&mut self, callback: F) -> Option<CallbackId> {
        match &mut self.t {
            Type::IC(_) => {
                panic!("callback cannot be added to input cell. ");
                unimplemented!()
            }
            Type::CC(cct) => cct.add_callback(callback),
        }
    }

    pub fn remove_callback(&mut self, callback: CallbackId) -> Result<(), RemoveCallbackError> {
        match &mut self.t {
            Type::IC(_) => {
                panic!("callback cannot be removed from input cell. ");
                unimplemented!()
            }
            Type::CC(cct) => cct.remove_callback(callback),
        }
    }

    fn calculate_new_value(&mut self, current_child_value: T) {
        let mut updated_value = None;
        match &mut self.t {
            Type::IC(ict) => { /*invalid*/ }
            Type::CC(cct) => {
                let mut values = vec![];
                for child in cct.children_iter() {
                    if let Ok(c) = child.try_borrow() {
                        let v = c.node_value;
                        values.push(v);
                    } else {
                        values.push(current_child_value);
                    }
                }
                let new_value = cct.rerun_compute_function(values.as_slice());
                if new_value != self.node_value {
                    updated_value = Some(new_value); // I would love to call self.set_value() here but borrower doesnt like it.
                    cct.invoke_callbacks(new_value);
                }
            }
        }
        if let Some(new_value) = updated_value {
            self.set_value(new_value);
        }
    }
}
