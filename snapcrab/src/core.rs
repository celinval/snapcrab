//! Core interpreter components that don't depend on rustc_private
//! This module can be tested in integration tests

pub mod stack {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Value {
        Int(i128),
        Uint(u128),
        Bool(bool),
        Unit,
    }
}

pub mod heap {
    use std::alloc::Layout;
    use std::collections::HashMap;
    use crate::core::stack::Value;

    #[derive(Debug)]
    pub struct Heap {
        storage: HashMap<usize, (Value, Layout)>,
        next_id: usize,
    }

    impl Heap {
        pub fn new() -> Self {
            Self {
                storage: HashMap::new(),
                next_id: 0,
            }
        }

        pub fn allocate(&mut self, value: Value, layout: Layout) -> usize {
            let id = self.next_id;
            self.next_id += 1;
            self.storage.insert(id, (value, layout));
            id
        }

        pub fn get(&self, id: usize) -> Option<&Value> {
            self.storage.get(&id).map(|(value, _)| value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_operations() {
        let int_val = stack::Value::Int(42);
        let uint_val = stack::Value::Uint(100);
        let bool_val = stack::Value::Bool(true);
        let unit_val = stack::Value::Unit;

        assert_eq!(int_val, stack::Value::Int(42));
        assert_eq!(uint_val, stack::Value::Uint(100));
        assert_eq!(bool_val, stack::Value::Bool(true));
        assert_eq!(unit_val, stack::Value::Unit);
    }

    #[test]
    fn test_heap_operations() {
        let mut heap = heap::Heap::new();
        let layout = std::alloc::Layout::new::<i32>();
        
        let id = heap.allocate(stack::Value::Int(42), layout);
        assert_eq!(heap.get(id), Some(&stack::Value::Int(42)));
        assert_eq!(heap.get(999), None);
    }
}
