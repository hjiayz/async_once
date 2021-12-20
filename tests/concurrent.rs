#[cfg(not(target_arch = "wasm32"))]
mod concurrent {
    use std::time::Duration;

    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use tokio::runtime::Runtime;

    lazy_static! {
        static ref FOO: AsyncOnce<u32> = AsyncOnce::new(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            1
        });
    }

    /// this test triggers a deadlock the test never commpletes
    #[test]
    fn simultaneous_access() {
        let child = std::thread::spawn(|| {
            Runtime::new()
                .unwrap()
                .block_on(async { assert_eq!(FOO.get().await, &1) });
        });

        Runtime::new()
            .unwrap()
            .block_on(async { assert_eq!(FOO.get().await, &1) });

        child.join().unwrap();
    }
}
