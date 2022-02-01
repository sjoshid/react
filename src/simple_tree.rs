use crate::CellId;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

enum Type<T> {
    IC(Option<Vec<Weak<RefCell<Node<T>>>>>), // just parents.
    CC(
        Option<Vec<Weak<RefCell<Node<T>>>>>,
        Option<Vec<Rc<RefCell<Node<T>>>>>,
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
    node_value: T,
    t: Type<T>,
}

impl<T: Copy + Debug> Node<T> {
    pub fn create_input(value: T) -> Self {
        Self {
            node_value: value,
            t: Type::IC(Some(vec![])),
        }
    }

    pub fn create_compute<F: 'static + Fn(&[T]) -> T>(
        children: Option<Vec<Rc<RefCell<Node<T>>>>>,
        compute_func: F,
    ) -> Result<Rc<RefCell<Node<T>>>, CellId> {
        let vals: Vec<T> = children
            .iter()
            .flat_map(|v| v.iter().map(|e| e.borrow().node_value))
            .collect();
        let vals = vals.as_slice();
        let value = compute_func(vals);

        let c1 = children.clone();
        let cc = Self {
            node_value: value,
            t: Type::CC(Some(vec![]), children, Rc::new(compute_func)),
        };
        let rc = Rc::new(RefCell::new(cc));

        if let Some(children) = c1 {
            children.iter().for_each(|c| match &mut c.borrow_mut().t {
                Type::IC(p) => {
                    if let Some(parent) = p.as_mut() {
                        let w = rc.clone();
                        parent.push(Rc::downgrade(&w));
                    }
                }
                Type::CC(p, _c, _cf) => {
                    if let Some(parent) = p.as_mut() {
                        let w = rc.clone();
                        parent.push(Rc::downgrade(&w));
                    }
                }
            })
        }

        Ok(rc)
    }

    pub fn set_value(&mut self, value: T) {
        match &mut self.t {
            Type::IC(p) => {
                self.node_value = value;
                if let Some(parents) = p.as_mut() {
                    parents.iter().for_each(|p| {
                        if let Some(n) = p.upgrade() {
                            let mut mut_borrow_parent = n.borrow_mut();
                            match &mut_borrow_parent.t {
                                Type::IC(_) => {
                                    panic!("IC cannot be parent!")
                                }
                                Type::CC(_, c, cf) => {
                                    let mut values = vec![];
                                    if let Some(children) = c.as_ref() {
                                        children.iter().for_each(|c| {
                                            if c.try_borrow().is_ok() {
                                                values.push(c.borrow().node_value);
                                            } else {
                                                values.push(value);
                                            }
                                        })
                                    }
                                    let new_value = cf(values.as_slice());
                                    mut_borrow_parent.node_value = new_value;
                                }
                            }
                            mut_borrow_parent.set_value(value);
                        }
                    })
                }
            }
            Type::CC(p, c, cf) => {
                let mut values = vec![];
                if let Some(children) = c.as_ref() {
                    children.iter().for_each(|c| {
                        values.push(c.borrow().node_value);
                    })
                }
                self.node_value = cf(values.as_slice());

                if let Some(parents) = p.as_mut() {
                    parents.iter_mut().for_each(|p| {
                        if let Some(n) = p.upgrade() {
                            match &n.borrow_mut().t {
                                Type::IC(_) => {
                                    panic!("IC cannot be parent!")
                                }
                                Type::CC(_, c, cc_cf) => {
                                    let mut values = vec![];
                                    if let Some(children) = c.as_ref() {
                                        children.iter().for_each(|c| {
                                            if c.try_borrow().is_ok() {
                                                values.push(c.borrow().node_value);
                                            } else {
                                                //values.push(self.node_value);
                                            }
                                        })
                                    }
                                    n.borrow_mut().node_value = cc_cf(values.as_slice());
                                }
                            }
                            n.borrow_mut().set_value(value);
                        }
                    })
                }
            }
        }
    }

    pub fn get_value(&self) -> T {
        self.node_value
    }
}
