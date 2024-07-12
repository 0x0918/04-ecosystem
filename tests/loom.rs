use loom::{
    sync::{atomic::AtomicUsize, Arc},
    thread::spawn,
};

#[test]
#[should_panic]
fn buggy_concurrent_inc() {
    loom::model(|| {
        let num = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..2)
            .map(|_| {
                let num = num.clone();
                spawn(move || {
                    let curr = num.load(std::sync::atomic::Ordering::Acquire);
                    num.store(curr + 1, std::sync::atomic::Ordering::Release);
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(2, num.load(std::sync::atomic::Ordering::Relaxed));
    });
}
