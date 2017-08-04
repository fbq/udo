use std::sync::atomic::{AtomicPtr, AtomicBool, Ordering};
use std::ops::{Deref, DerefMut};
use std::boxed::Box;
use std::cell::UnsafeCell;
use std::ptr;

struct MCSNode {
    locked : AtomicBool,
    next : AtomicPtr<MCSNode>,
}

impl Drop for MCSNode {
    fn drop(&mut self) {
        #[cfg(test)] eprintln!("MCSNode {:?} freed", &*self as *const MCSNode);
    }
}


struct MCSQueue {
    tail : AtomicPtr<MCSNode>,
}

unsafe fn mcs_lock(queue: *mut MCSQueue, node : *mut MCSNode) {
    #[cfg(test)] eprintln!("Locked at {:?} in queue {:?}", node, queue);

    let prev = (*queue).tail.swap(node, Ordering::AcqRel);

    if prev.is_null() {
        return;
    }

    (*prev).next.store(node, Ordering::Relaxed);


    while (*node).locked.load(Ordering::Acquire) {
    }
}

unsafe fn mcs_unlock(queue: *mut MCSQueue, node : *mut MCSNode) {
    #[cfg(test)] eprintln!("Unlocked at {:?} in queue {:?}", node, queue);

    if (*queue).tail.compare_and_swap(node, ptr::null_mut(), Ordering::AcqRel) == node {
        return;
    }

    loop {
        let next = (*node).next.load(Ordering::Relaxed);

        if !next.is_null() {
            (*next).locked.store(false, Ordering::Release);
            return;
        }
    }
}

pub struct LockGuard<'a, T> {
    data : *mut T,
    queue : &'a UnsafeCell<MCSQueue>,
    node_ptr : *mut MCSNode,
}

impl<'a, T> Drop for LockGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            mcs_unlock(self.queue.get(), self.node_ptr);

            // Free the object create by box
            Box::from_raw(self.node_ptr);
        }
    }

}


impl<'a, T> Deref for LockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &*self.data
        }
    }

}

impl<'a, T> DerefMut for LockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T{
        unsafe {
            &mut *self.data
        }
    }
}

pub struct Lock<T> {
    queue : UnsafeCell<MCSQueue>,
    data : UnsafeCell<T>,
}

impl<T> Lock<T> {
    pub fn new(t : T) -> Lock<T> {
        Lock {
            queue : UnsafeCell::new(MCSQueue {
                tail : AtomicPtr::new(ptr::null_mut()),
            }),
            data : UnsafeCell::new(t),
        }
    }

    pub fn lock(&self) -> LockGuard<T> {
        let node_box = Box::new(MCSNode {
                locked : AtomicBool::new(true),
                next : AtomicPtr::new(ptr::null_mut()),
            });

        let ret = LockGuard {
            node_ptr : Box::into_raw(node_box),
            data : self.data.get(),
            queue : & self.queue,
        };

        unsafe {
            mcs_lock(ret.queue.get(), ret.node_ptr);
        }
        ret
    }
}

unsafe impl<T> Sync for Lock<T> {}
unsafe impl<T> Send for Lock<T> {}

