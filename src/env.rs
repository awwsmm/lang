use std::collections::HashMap;
use crate::val::Val;

pub(crate) struct Env {
    bindings: HashMap<String, Val>,
}

impl Env {
    pub(crate) fn store_binding(&mut self, name: String, val: Val) {
        self.bindings.insert(name, val);
    }
}