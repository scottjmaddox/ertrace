//! Singly linked list of ErtraceLocations, backed by either a global, static
//! block of
//! memory or by blocks of memory provided by the global allocator,
//! with lock-free O(1) Drop to a global free list.

use crate::ertrace_location::ErtraceLocation;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};

static FREE_LIST: AtomicPtr<ErtraceNode> = AtomicPtr::new(core::ptr::null_mut());

fn try_take_from_free_list() -> Result<NonNull<ErtraceNode>, ()> {
    loop {
        let current_ptr = FREE_LIST.load(Ordering::SeqCst);
        let node = if let Some(p) = unsafe { current_ptr.as_mut() } {
            p
        } else {
            return Err(());
        };
        let new_ptr = node.next.load(Ordering::SeqCst);
        if FREE_LIST.compare_and_swap(current_ptr, new_ptr, Ordering::SeqCst) == current_ptr {
            node.next.store(core::ptr::null_mut(), Ordering::SeqCst);
            return Ok(unsafe { NonNull::new_unchecked(node) });
        }
    }
}

fn new_tail_node(data: &'static ErtraceLocation) -> NonNull<ErtraceNode> {
    match try_take_from_free_list() {
        Ok(mut node_ptr) => {
            unsafe { node_ptr.as_mut() }.data = data;
            node_ptr
        }
        Err(()) => {
            //TODO: alloc from arena
            let node_ptr = Box::into_raw(Box::new(ErtraceNode {
                next: AtomicPtr::new(core::ptr::null_mut()),
                data,
            }));
            unsafe { NonNull::new_unchecked(node_ptr) }
        }
    }
}

#[derive(Debug)]
pub struct Ertrace {
    head: NonNull<ErtraceNode>,
    tail: NonNull<ErtraceNode>,
}

impl core::fmt::Display for Ertrace {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        writeln!(f, "error return trace:")?;
        for (i, location) in self.iter().enumerate() {
            write!(f, "{:>5}: {}", i, location)?;
        }
        writeln!(f, "")
    }
}

impl Ertrace {
    pub fn new(data: &'static ErtraceLocation) -> Self {
        let new_tail = new_tail_node(data);
        Self {
            head: new_tail,
            tail: new_tail,
        }
    }

    pub fn from_cause<T: Into<Ertrace>>(cause: T, data: &'static ErtraceLocation) -> Self {
        let mut ertrace: crate::Ertrace = cause.into();
        ertrace.push_back(data);
        ertrace
    }

    pub fn push_back(&mut self, data: &'static ErtraceLocation) {
        let new_tail = new_tail_node(data);
        unsafe { self.tail.as_mut() }
            .next
            .store(new_tail.as_ptr(), Ordering::SeqCst);
        self.tail = new_tail;
    }

    pub fn iter(&self) -> ErtraceIter {
        ErtraceIter {
            ertrace: &self,
            maybe_next_ptr: Some(self.head),
        }
    }
}

#[derive(Debug)]
pub struct ErtraceIter<'a> {
    #[allow(dead_code)]
    ertrace: &'a Ertrace,
    maybe_next_ptr: Option<NonNull<ErtraceNode>>,
}

impl<'a> Iterator for ErtraceIter<'a> {
    type Item = &'static ErtraceLocation;
    fn next(&mut self) -> Option<Self::Item> {
        match self.maybe_next_ptr {
            Some(next_ptr) => {
                let node = unsafe { next_ptr.as_ref() };
                self.maybe_next_ptr = NonNull::new(node.next.load(Ordering::SeqCst));
                Some(node.data)
            }
            None => None,
        }
    }
}

#[derive(Debug)]
struct ErtraceNode {
    next: AtomicPtr<ErtraceNode>,
    data: &'static ErtraceLocation,
}

impl Drop for Ertrace {
    fn drop(&mut self) {
        let tail = unsafe { self.tail.as_mut() };
        loop {
            let old_next = FREE_LIST.load(Ordering::SeqCst);
            tail.next.store(old_next, Ordering::SeqCst);
            if FREE_LIST.compare_and_swap(old_next, self.head.as_ptr(), Ordering::SeqCst)
                == old_next
            {
                return;
            }
        }
    }
}
