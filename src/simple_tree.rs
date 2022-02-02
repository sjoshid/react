use crate::{CellId, ComputeCellType, InputCellType};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

enum Type<T> {
    IC(InputCellType<T>),
    CC(ComputeCellType<T>),
}

pub struct Node<T> {
    node_value: T,
    t: Type<T>,
}

impl<T: Copy + Debug + PartialEq> Node<T> {
    pub fn create_input(value: T) -> Self {
        Self {
            node_value: value,
            t: Type::IC(InputCellType::new(vec![])),
        }
    }

    pub fn create_compute<F: 'static + Fn(&[T]) -> T>(
        children: Vec<Rc<RefCell<Node<T>>>>,
        compute_func: F,
    ) -> Result<Rc<RefCell<Node<T>>>, CellId> {
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
                for parent in ict.into_iter() {
                    parent.borrow_mut().calculate_new_value(value);
                }
            }
            Type::CC(cct) => {
                for parent in cct.into_iter() {
                    parent.borrow_mut().calculate_new_value(value);
                }
            }
        }
    }

    pub fn get_value(&self) -> T {
        self.node_value
    }

    fn calculate_new_value(&mut self, current_child_value: T) {
        match &self.t {
            Type::IC(ict) => { /*invalid*/ }
            Type::CC(cct) => {
                let mut values = vec![];
                for child in cct.into_iter() {
                    if let Ok(c) = child.try_borrow() {
                        let v = c.node_value;
                        values.push(v);
                    } else {
                        values.push(current_child_value);
                    }
                }
                let new_value = cct.rerun_compute_function(values.as_slice());
                if new_value != self.node_value {
                    self.set_value(new_value);
                    //sj_todo do something will callback here?
                }
            }
        }
    }
}

/*match &mut self.t {
    Type::IC(p) => {
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
                            if mut_borrow_parent.node_value != new_value {
                                mut_borrow_parent.set_value(new_value);
                            }
                        }
                    }
                }
            })
        }
    }
    Type::CC(p, c, cf) => {
        if let Some(parents) = p.as_mut() {
            parents.iter().for_each(|p| {
                if let Some(n) = p.upgrade() {
                    let mut mut_borrow_parent = n.borrow_mut();
                    match &mut_borrow_parent.t {
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
                                        values.push(value);
                                    }
                                })
                            }
                            let new_value = cc_cf(values.as_slice());
                            if mut_borrow_parent.node_value != new_value {
                                mut_borrow_parent.set_value(new_value);
                            }
                        }
                    }
                }
            })
        }
    }
}*/

/*struct ComputeCellType<T> {
    parents: Vec<Weak<RefCell<Node<T>>>>,
    children: Vec<Rc<RefCell<Node<T>>>>,
    compute_function: Rc<dyn Fn(&[T]) -> T>,
    //callback_function:
}*/

/*impl<T: Copy> Clone for Type<T> {
    fn clone(&self) -> Self {
        match self {
            Type::IC(p) => Type::IC(p.clone()),
            Type::CC(p, c, cf) => Type::CC(p.clone(), c.clone(), cf.clone()),
        }
    }
}*/
