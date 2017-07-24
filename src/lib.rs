mod mcs;

pub use mcs::{Lock, LockGuard};

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {

        use mcs::Lock;
        use std::thread;
        use std::sync::Arc;

        let m = Arc::new(Lock::new(0));

        let m1 = m.clone();
        let m2 = m.clone();

        {
            let l = m.lock();

            println!("{:?}", *l);
        }

        let t2 = thread::spawn(move || {
            let l = m2.lock();
            println!("{:?}", *l);
        });

        let t1 = thread::spawn(move || {
            let mut l = m1.lock();
            *l = 1;
            println!("{:?}", *l);
        });

        t1.join().unwrap();
        t2.join().unwrap();

    }
}
