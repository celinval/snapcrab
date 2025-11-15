use crate::stack::Value;
use std::alloc::Layout;

pub type Address = usize;

#[derive(Debug, Clone)]
struct HeapEntry {
    value: Value,
    layout: Layout,
}

#[derive(Debug)]
pub struct Heap {
    memory: Vec<Option<HeapEntry>>,
    next_address: Address,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            memory: Vec::new(),
            next_address: 0,
        }
    }

    #[allow(dead_code)]
    pub fn allocate(&mut self, value: Value, layout: Layout) -> Address {
        let address = self.next_address;
        if address >= self.memory.len() {
            self.memory.resize(address + 1, None);
        }
        self.memory[address] = Some(HeapEntry { value, layout });
        self.next_address += 1;
        address
    }

    #[allow(dead_code)]
    pub fn read(&self, address: Address) -> Option<Value> {
        self.memory.get(address)
            .and_then(|entry| entry.as_ref().map(|e| e.value))
    }

    #[allow(dead_code)]
    pub fn write(&mut self, address: Address, value: Value) -> bool {
        if let Some(Some(entry)) = self.memory.get_mut(address) {
            entry.value = value;
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn get_layout(&self, address: Address) -> Option<Layout> {
        self.memory.get(address)
            .and_then(|entry| entry.as_ref().map(|e| e.layout))
    }
}
