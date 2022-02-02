extern crate core;

mod simple_tree;

pub use simple_tree::Node;

use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

/// `InputCellId` is a unique identifier for an input cell.
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub struct InputCellId {
    id: usize,
}

impl InputCellId {
    fn new(index: usize) -> InputCellId {
        let id = Self { id: index };
        id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub struct ComputeCellId {
    id: usize,
}

impl ComputeCellId {
    fn new(index: usize) -> ComputeCellId {
        let id = Self { id: index };
        id
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CallbackId();

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum CellId {
    Input(InputCellId),
    Compute(ComputeCellId),
}

#[derive(Debug, PartialEq)]
pub enum RemoveCallbackError {
    NonexistentCell,
    NonexistentCallback,
}

impl From<CellId> for InputCellId {
    fn from(ic: CellId) -> Self {
        match ic {
            CellId::Input(ic) => ic,
            CellId::Compute(_) => {
                panic!("Invalid. ic expected found cc.")
            }
        }
    }
}

impl From<CellId> for ComputeCellId {
    fn from(ic: CellId) -> Self {
        match ic {
            CellId::Input(_) => {
                panic!("Invalid. cc expected found ic")
            }
            CellId::Compute(cc) => cc,
        }
    }
}

impl From<CellId> for usize {
    fn from(cell_id: CellId) -> Self {
        match cell_id {
            CellId::Input(ic) => ic.id,
            CellId::Compute(cc) => cc.id,
        }
    }
}

pub struct Reactor<T> {
    store: Vec<Rc<RefCell<Node<T>>>>,
    counter: usize,
}

// You are guaranteed that Reactor will only be tested against types that are Copy + PartialEq.
impl<T: Copy + Debug + PartialEq> Reactor<T> {
    //sj_todo what is T is not copyable?
    pub fn new() -> Self {
        Self {
            store: vec![],
            counter: 0,
        }
    }

    // Creates an input cell with the specified initial value, returning its ID.
    pub fn create_input(&mut self, initial: T) -> InputCellId {
        let index = self.counter;

        let ic = Node::create_input(initial);
        let rc_p1 = Rc::new(RefCell::new(ic));
        self.store.insert(index, rc_p1.clone());
        self.counter = self.counter + 1;
        InputCellId::new(index)
    }

    pub fn create_compute<F: 'static + Fn(&[T]) -> T>(
        &mut self,
        dependencies: &[CellId],
        compute_func: F,
    ) -> Result<ComputeCellId, CellId> {
        let mut deps = Vec::with_capacity(dependencies.len());

        for cell_id in dependencies {
            match cell_id {
                CellId::Input(ic) => {
                    if let Some(n) = self.store.get(ic.id) {
                        deps.push(n.clone());
                    } else {
                        return Err(*cell_id);
                    }
                }
                CellId::Compute(cc) => {
                    if let Some(n) = self.store.get(cc.id) {
                        deps.push(n.clone());
                    } else {
                        return Err(*cell_id);
                    }
                }
            }
        }

        let index = self.counter;
        if let Ok(cc) = Node::create_compute(Some(deps), compute_func) {
            self.store.insert(index, cc.clone());
            self.counter = self.counter + 1;
        }
        Ok(ComputeCellId::new(index))
        /*let cc = ComputeCellId::new();
        let mut dependent_on = vec![];
        let deps: Vec<T> = dependencies
            .iter()
            .map(|c| {
                let index: usize = (*c).into();
                dependent_on.push(index);
                let d = self.store.get(index).unwrap(); //sj_todo need to handle error while unwrapping
                d.value
            })
            .collect();
        let deps = deps.as_slice();
        let value = compute_func(deps);
        let d = Detail {
            value,
            required_by: None,
            dependent_on: Some(dependent_on),
            compute_function: Some(Box::new(compute_func)),
        };
        self.store.insert(cc.into(), d);

        for i in dependencies.iter() {
            let index: usize = (*i).into();
            let d = self.store.get_mut(index).unwrap();
            if let Some(v) = &mut d.required_by {
                v.push(cc.into());
            } else {
                let mut v = vec![cc.into()];
                d.required_by = Some(v);
            }
        }
        Ok(cc.into())*/
    }

    // Retrieves the current value of the cell, or None if the cell does not exist.
    //
    // You may wonder whether it is possible to implement `get(&self, id: CellId) -> Option<&Cell>`
    // and have a `value(&self)` method on `Cell`.
    //
    // It turns out this introduces a significant amount of extra complexity to this exercise.
    // We chose not to cover this here, since this exercise is probably enough work as-is.
    pub fn value(&self, id: CellId) -> Option<T> {
        let index: usize = id.into();
        if let Some(v) = self.store.get(index) {
            let value = v.as_ref().borrow().get_value();
            Some(value)
        } else {
            None
        }
    }

    // Sets the value of the specified input cell.
    //
    // Returns false if the cell does not exist.
    //
    // Similarly, you may wonder about `get_mut(&mut self, id: CellId) -> Option<&mut Cell>`, with
    // a `set_value(&mut self, new_value: T)` method on `Cell`.
    //
    // As before, that turned out to add too much extra complexity.
    pub fn set_value(&mut self, ic: InputCellId, new_value: T) -> bool {
        let index: usize = ic.id;
        if let Some(v) = self.store.get(index) {
            let mut node = v.as_ref().borrow_mut();
            node.set_value(new_value);
            true
        } else {
            false
        }
    }

    // Adds a callback to the specified compute cell.
    //
    // Returns the ID of the just-added callback, or None if the cell doesn't exist.
    //
    // Callbacks on input cells will not be tested.
    //
    // The semantics of callbacks (as will be tested):
    // For a single set_value call, each compute cell's callbacks should each be called:
    // * Zero times if the compute cell's value did not change as a result of the set_value call.
    // * Exactly once if the compute cell's value changed as a result of the set_value call.
    //   The value passed to the callback should be the final value of the compute cell after the
    //   set_value call.
    pub fn add_callback<F: FnMut(T)>(
        &mut self,
        _id: ComputeCellId,
        _callback: F,
    ) -> Option<CallbackId> {
        unimplemented!()
    }

    // Removes the specified callback, using an ID returned from add_callback.
    //
    // Returns an Err if either the cell or callback does not exist.
    //
    // A removed callback should no longer be called.
    pub fn remove_callback(
        &mut self,
        cell: ComputeCellId,
        callback: CallbackId,
    ) -> Result<(), RemoveCallbackError> {
        unimplemented!(
            "Remove the callback identified by the CallbackId {:?} from the cell {:?}",
            callback,
            cell,
        )
    }
}

/*
Following function are not used.


    fn set_new_value(&mut self, ic: InputCellId, new_value: T) -> Option<Vec<usize>> {
        /*let index = ic.id;
        if let Some(d) = self.store.get_mut(index) {
            d.value = new_value;
            d.required_by.clone()
        } else {
            None
        }*/
        todo!()
    }

    fn calculate_new_value(&self, detail: &mut Detail<T>) {
        /*if let Some(cf) = detail.compute_function.as_ref() {
            if let Some(v) = detail.dependent_on.as_ref() {
                let values: Vec<T> = v
                    .iter()
                    .map(|e| self.store.get(*e).unwrap().value)
                    .collect();
                detail.value = cf(values.as_slice());
            }
        }*/
        todo!()
    }
 */
/*struct Detail<T> {
    //id: CellId,
    value: T,
    required_by: Option<Vec<usize>>,
    dependent_on: Option<Vec<usize>>,
    compute_function: Option<Box<dyn Fn(&[T]) -> T>>,
}

impl<T: Copy> Detail<T> {
    fn get_detail_reqs<'a, 'b: 'a>(
        &self,
        store: &'b mut Vec<Detail<T>>,
    ) -> Option<Vec<&'a mut Detail<T>>> {
        if let Some(v) = self.required_by.as_ref() {
            let mut hm: HashMap<_, _> = store.into_iter().enumerate().collect();
            let r = v.into_iter().flat_map(|index| hm.remove(index)).collect();
            Some(r)
        } else {
            None
        }
    }
}*/
