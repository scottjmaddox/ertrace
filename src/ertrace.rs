//! Singly linked list of `ErtraceLocation`s, with O(1) Drop to a global,
//! lock-free free list.

extern crate alloc;

use crate::ertrace_location::ErtraceLocation;
use alloc::boxed::Box;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};

static FREE_LIST: AtomicPtr<ErtraceNode> = AtomicPtr::new(core::ptr::null_mut());

fn try_take_from_free_list() -> Result<NonNull<ErtraceNode>, ()> {
    loop {
        let current_ptr = FREE_LIST.load(Ordering::Acquire);
        // NOTE: this is safe because this pointer is either null or valid.
        let node = if let Some(p) = unsafe { current_ptr.as_ref() } {
            p
        } else {
            return Err(());
        };
        let new_ptr = node.next.load(Ordering::Relaxed);
        if FREE_LIST.compare_and_swap(current_ptr, new_ptr, Ordering::Release) == current_ptr {
            node.next.store(core::ptr::null_mut(), Ordering::Relaxed);
            // NOTE: this is safe because the pointer is non-null if we reached here.
            return Ok(unsafe { NonNull::new_unchecked(current_ptr) });
        }
    }
}

fn new_tail_node(data: &'static ErtraceLocation) -> NonNull<ErtraceNode> {
    match try_take_from_free_list() {
        Ok(mut node_ptr) => {
            // NOTE: this is safe because the pointer is valid and globally unique.
            unsafe { node_ptr.as_mut() }.data = data;
            node_ptr
        }
        Err(()) => {
            //TODO: alloc from arena
            let node_ptr = Box::into_raw(Box::new(ErtraceNode {
                next: AtomicPtr::new(core::ptr::null_mut()),
                data,
            }));
            // NOTE: this is safe because the pointer is non-null.
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
        // NOTE: this is safe because
        //   1) the pointer to this ErtraceNode is valid, and
        //   2) we control access to the only other pointer to this ErtraceNode,
        //   and can guarantee that it is not converted into a mutable reference while
        //   this mutable reference exists.
        unsafe { self.tail.as_mut() }
            .next
            .store(new_tail.as_ptr(), Ordering::Relaxed);
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
                // NOTE: this is safe because the pointer is valid.
                let node = unsafe { next_ptr.as_ref() };
                self.maybe_next_ptr = NonNull::new(node.next.load(Ordering::Relaxed));
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
        // NOTE: this is safe because
        //   1) the pointer to this ErtraceNode is valid, and
        //   2) we control access to the only other pointer to this ErtraceNode,
        //   and can guarantee that it is not converted into a mutable reference while
        //   this mutable reference exists.
        let tail = unsafe { self.tail.as_mut() };
        loop {
            let old_next = FREE_LIST.load(Ordering::Acquire);
            tail.next.store(old_next, Ordering::Relaxed);
            if FREE_LIST.compare_and_swap(old_next, self.head.as_ptr(), Ordering::Release)
                == old_next
            {
                return;
            }
        }
    }
}
