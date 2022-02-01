use crate::CellId;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

enum Type<T> {
    IC(Option<Vec<Weak<RefCell<Node<T>>>>>), // just parents.
    CC(
        Option<Vec<Weak<RefCell<Node<T>>>>>,
        Option<Vec<Rc<Node<T>>>>,
        Rc<dyn Fn(&[T]) -> T>,
    ), // parents, children and compute function
}

impl<T: Copy> Clone for Type<T> {
    fn clone(&self) -> Self {
        match self {
            Type::IC(p) => Type::IC(p.clone()),
            Type::CC(p, c, cf) => Type::CC(p.clone(), c.clone(), cf.clone()),
        }
    }
}

pub struct Node<T> {
    value: T,
    t: Type<T>,
}

impl<T: Copy> Node<T> {
    pub fn new_ic(value: T) -> Self {
        Self {
            value,
            t: Type::IC(None),
        }
    }

    pub fn create_compute<F: 'static + Fn(&[T]) -> T>(
        children: Option<Vec<Rc<Node<T>>>>,
        compute_func: F,
    ) -> Result<Node<T>, CellId> {
        let vals: Vec<T> = children
            .iter()
            .flat_map(|v| v.iter().map(|e| e.value))
            .collect();
        let vals = vals.as_slice();
        let value = compute_func(vals);

        let c1 = children.clone();
        let cc = Self {
            value,
            t: Type::CC(None, children, Rc::new(compute_func)),
        };

        Ok(cc)
    }

    pub fn set_value(&mut self, value: T) {
        match &mut self.t {
            Type::IC(p) => {
                self.value = value;
                if let Some(parents) = p.as_mut() {
                    parents.iter_mut().for_each(|p| {
                        if let Some(n) = p.upgrade() {
                            match &n.borrow_mut().t {
                                Type::IC(_) => { /*invalid*/ }
                                Type::CC(_, c, cf) => {
                                    let mut values = vec![];
                                    if let Some(children) = c.as_ref() {
                                        children.iter().for_each(|c| {
                                            values.push(c.value);
                                        })
                                    }
                                    let new_value = cf(values.as_slice());
                                    n.borrow_mut().value = new_value;
                                }
                            }
                            n.borrow_mut().set_value(value);
                        }
                    })
                }
            }
            Type::CC(p, c, cf) => {
                let mut values = vec![];
                if let Some(children) = c.as_ref() {
                    children.iter().for_each(|c| {
                        values.push(c.value);
                    })
                }
                self.value = cf(values.as_slice());

                if let Some(parents) = p.as_mut() {
                    parents.iter_mut().for_each(|p| {
                        if let Some(n) = p.upgrade() {
                            match &n.borrow_mut().t {
                                Type::IC(_) => { /*invalid*/ }
                                Type::CC(_, c, cc_cf) => {
                                    let mut values = vec![];
                                    if let Some(children) = c.as_ref() {
                                        children.iter().for_each(|c| {
                                            values.push(c.value);
                                        })
                                    }
                                    n.borrow_mut().value = cc_cf(values.as_slice());
                                }
                            }
                            n.borrow_mut().set_value(value);
                        }
                    })
                }
            }
        }
    }
}
