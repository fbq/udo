use std::sync::atomic::{AtomicPtr, Ordering};
use std::ops::{Deref, DerefMut};
use std::boxed::Box;
use std::ptr;

struct MCSNode {
    locked : bool,
    next : AtomicPtr<MCSNode>,
}

impl Drop for MCSNode {
    fn drop(&mut self) {
        println!("MCSNode freed"); 
    }
}


struct MCSQueue {
    tail : AtomicPtr<MCSNode>,
}

unsafe fn mcs_lock(queue: *mut MCSQueue, node : *mut MCSNode) {
    let prev = (*queue).tail.swap(node, Ordering::SeqCst);

    if prev.is_null() {
        return;
    }

    (*prev).next.store(node, Ordering::SeqCst);


    while (*node).locked {
    }
}

unsafe fn mcs_unlock(queue: *mut MCSQueue, node : *mut MCSNode) {
    if (*queue).tail.compare_and_swap(node, ptr::null_mut(), Ordering::SeqCst) == node {
        return; 
    }

    while (*node).next.load(Ordering::SeqCst).is_null() {}

    (*(*node).next.load(Ordering::SeqCst)).locked = false;
}

pub struct LockGuard<T> {
    data : *mut T,
    queue : *const MCSQueue,
    node_ptr : *mut MCSNode,
}

impl<T> Drop for LockGuard<T> {
    fn drop(&mut self) {
        unsafe {
            mcs_unlock(self.queue as *mut MCSQueue, self.node_ptr);

            // Free the object create by box
            Box::from_raw(self.node_ptr);
        }
    }

}


impl<T> Deref for LockGuard<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { 
            &*self.data
        }
    }

}

impl<T> DerefMut for LockGuard<T> {
    fn deref_mut(&mut self) -> &mut T{
        unsafe { 
            &mut *self.data
        }
    }
}

pub struct Lock<T> {
    queue : MCSQueue,
    data_ptr : *mut T, 
}

impl<T> Lock<T> {
    pub fn new(t : T) -> Lock<T> {
        let data = Box::new(t);

        Lock {
            queue : MCSQueue {
                tail : AtomicPtr::new(ptr::null_mut()),
            },
            data_ptr : Box::into_raw(data),
        }
    }

    pub fn lock(&self) -> LockGuard<T> {
        let node_box = Box::new(MCSNode {
                locked : true,
                next : AtomicPtr::new(ptr::null_mut()),
            });

        let ret = LockGuard {
            node_ptr : Box::into_raw(node_box),
            data : self.data_ptr,
            queue : & self.queue,
        };

        unsafe {
            mcs_lock(ret.queue as *mut MCSQueue, ret.node_ptr);
        }
        ret
    }
}

unsafe impl<T> Sync for Lock<T> {}
unsafe impl<T> Send for Lock<T> {}

