mod mcs;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {

        use mcs;
        use std::thread;
        use std::sync::Arc;

        let m = Arc::new(mcs::Lock::new(0));

        let m1 = m.clone();
        let m2 = m.clone();

        {
            let l = m.lock();

            println!("main thread {:?}!!!", *l);
        }

        let t2 = thread::spawn(move || {
            let l = m2.lock();
            println!("read thread {:?}", *l);
        });

        let t1 = thread::spawn(move || {
            let mut l = m1.lock();
            *l = 1;
            println!("write thread1 {:?}", *l);
        });

        t1.join().unwrap();
        t2.join().unwrap();

    }
/*
 * This should introduce failure at compile time.
    #[test]
    fn lifetime() {
        use mcs::Lock;
        let mut l;

        {
            let m = Lock::new(0);
            l = m.lock();
        }

        *l = 1;
    }
*/

}
