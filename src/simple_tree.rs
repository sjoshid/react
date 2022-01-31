use crate::CellId;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

enum Type<T> {
    IC(Option<Vec<Weak<Node<T>>>>), // just parents.
    CC(
        Option<Vec<Weak<Node<T>>>>,
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

struct Node<T> {
    value: T,
    t: Type<T>,
}

impl<T: Copy> Node<T> {
    fn new_ic(value: T) -> Self {
        Self {
            value,
            t: Type::IC(None),
        }
    }

    pub fn create_compute<F: 'static + Fn(&[T]) -> T>(
        &mut self,
        children: Option<Vec<Rc<Node<T>>>>,
        compute_func: F,
    ) -> Result<Node<T>, CellId> {
        let vals: Vec<T> = children
            .iter()
            .flat_map(|v| v.iter().map(|e| e.value))
            .collect();
        let vals = vals.as_slice();
        let value = compute_func(vals);

        let cc = Self {
            value,
            t: Type::CC(None, children, Rc::new(compute_func)),
        };

        Ok(cc)
    }

    fn set_value(&mut self, value: T) {
        self.value = value;

        match &mut self.t {
            Type::IC(p) => {
                if let Some(parents) = p.as_mut() {
                    parents.iter_mut().for_each(|p| {
                        if let Some(n) = p.upgrade() {
                            match &n.t {
                                Type::IC(_) => { /*invalid*/ }
                                Type::CC(_, c, cf) => {
                                    let mut values = vec![];
                                    if let Some(children) = c.as_ref() {
                                        children.iter().for_each(|c| {
                                            values.push(c.value);
                                        })
                                    }
                                    let new_value = cf(values.as_slice());
                                }
                            }
                        }
                    })
                }
            }
            Type::CC(p, c, cf) => {
                if let Some(parents) = p.as_mut() {
                    parents.iter_mut().for_each(|p| {
                        if let Some(n) = p.upgrade() {
                            match &n.t {
                                Type::IC(_) => { /*invalid*/ }
                                Type::CC(_, c, _) => {
                                    let mut values = vec![];
                                    if let Some(children) = c.as_ref() {
                                        children.iter().for_each(|c| {
                                            values.push(c.value);
                                        })
                                    }
                                    let new_value = cf(values.as_slice());
                                }
                            }
                        }
                    })
                }
            }
        }
        todo!()
    }
}
