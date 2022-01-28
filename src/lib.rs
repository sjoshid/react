use std::borrow::Borrow;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

/// `InputCellId` is a unique identifier for an input cell.
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub struct InputCellId {
    id: usize,
}

impl InputCellId {
    fn new() -> CellId {
        let id = Self { id: 0 };
        CellId::Input(id)
    }
}

/// `ComputeCellId` is a unique identifier for a compute cell.
/// Values of type `InputCellId` and `ComputeCellId` should not be mutually assignable,
/// demonstrated by the following tests:
///
/// ```compile_fail
/// let mut r = react::Reactor::new();
/// let input: react::ComputeCellId = r.create_input(111);
/// ```
///
/// ```compile_fail
/// let mut r = react::Reactor::new();
/// let input = r.create_input(111);
/// let compute: react::InputCellId = r.create_compute(&[react::CellId::Input(input)], |_| 222).unwrap();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub struct ComputeCellId {
    id: usize,
}

impl ComputeCellId {
    fn new() -> CellId {
        let id = Self { id: 0 };
        CellId::Compute(id)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CallbackId();

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum CellId {
    Input(InputCellId),
    Compute(ComputeCellId),
}

struct Detail<T> {
    //id: CellId,
    value: T,
    required_by: Option<Vec<usize>>,
    compute_function: Option<Box<dyn Fn(&[T]) -> T>>,
}

impl<T: Copy> Detail<T> {
    fn calculate_new_value(&mut self, store: &Vec<Detail<T>>) {
        if let Some(cf) = self.compute_function.borrow() {
            if let Some(v) = self.required_by.borrow() {
                let values: Vec<T> = v.iter().map(|e| store.get(*e).unwrap().value).collect();
                self.value = cf(values.as_slice());
            }
        }
    }

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
}

impl From<CellId> for usize {
    fn from(cell_id: CellId) -> Self {
        match cell_id {
            CellId::Input(ic) => ic.id,
            CellId::Compute(cc) => cc.id,
        }
    }
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

#[derive(Debug, PartialEq)]
pub enum RemoveCallbackError {
    NonexistentCell,
    NonexistentCallback,
}

pub struct Reactor<T> {
    store: Vec<Detail<T>>,
}

// You are guaranteed that Reactor will only be tested against types that are Copy + PartialEq.
impl<T: Copy> Reactor<T> {
    //sj_todo what is T is not copyable?
    pub fn new() -> Self {
        Self { store: vec![] }
    }

    // Creates an input cell with the specified initial value, returning its ID.
    pub fn create_input(&mut self, initial: T) -> InputCellId {
        let ic = InputCellId::new();
        let d = Detail {
            value: initial,
            required_by: None,
            compute_function: None,
        };
        self.store.insert(ic.into(), d);
        ic.into()
    }

    // Creates a compute cell with the specified dependencies and compute function.
    // The compute function is expected to take in its arguments in the same order as specified in
    // `dependencies`.
    // You do not need to reject compute functions that expect more arguments than there are
    // dependencies (how would you check for this, anyway?).
    //
    // If any dependency doesn't exist, returns an Err with that nonexistent dependency.
    // (If multiple dependencies do not exist, exactly which one is returned is not defined and
    // will not be tested)
    //
    // Notice that there is no way to *remove* a cell.
    // This means that you may assume, without checking, that if the dependencies exist at creation
    // time they will continue to exist as long as the Reactor exists.
    pub fn create_compute<F: 'static + Fn(&[T]) -> T>(
        &mut self,
        dependencies: &[CellId],
        compute_func: F,
    ) -> Result<ComputeCellId, CellId> {
        let cc = ComputeCellId::new();
        let deps: Vec<T> = dependencies
            .iter()
            .map(|c| {
                let index: usize = (*c).into();
                let d = self.store.get(index).unwrap();
                d.value
            })
            .collect(); //sj_todo need to handle error while unwrapping
        let deps = deps.as_slice();
        let value = compute_func(deps);
        let d = Detail {
            value,
            required_by: None,
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
        Ok(cc.into())
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
        let v = self.store.get(index);
        v.map(|d| d.value)
    }

    fn set_new_value(&mut self, ic: InputCellId, new_value: T) -> Option<Vec<usize>> {
        let index = ic.id;
        if let Some(d) = self.store.get_mut(index) {
            d.value = new_value;
            d.required_by.clone()
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
        // if ic doesnt exist in store we return false
        if let Some(indexes) = self.set_new_value(ic, new_value) {
            let details: Vec<&mut Detail<T>> = self
                .store
                .iter_mut()
                .enumerate()
                .filter_map(|(i, d)| if indexes.contains(&i) { Some(d) } else { None })
                .collect();

            todo!();
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
