use futures::channel::mpsc;
use futures::channel::oneshot;
#[allow(unused_imports)]
use futures::stream::StreamExt;
#[allow(unused_imports)]
use std::future::Future;

pub struct AsyncOnce<T> {
    sender: mpsc::UnboundedSender<oneshot::Sender<T>>,
}

impl<T: 'static + Clone> AsyncOnce<T> {
    #[cfg(all(
        feature = "tokio",
        not(feature = "async-std"),
        not(feature = "wasm-bindgen-futures")
    ))]
    pub fn new<F>(fut: F) -> AsyncOnce<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send,
    {
        let (tx, mut rx) = mpsc::unbounded::<oneshot::Sender<T>>();
        tokio::spawn(async move {
            let val = fut.await;
            while let Some(val_tx) = rx.next().await {
                let _ = val_tx.send(val.clone());
            }
        });
        AsyncOnce { sender: tx }
    }
    #[cfg(all(
        feature = "async-std",
        not(feature = "tokio"),
        not(feature = "wasm-bindgen-futures")
    ))]
    pub fn new<F>(fut: F) -> AsyncOnce<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send,
    {
        let (tx, mut rx) = mpsc::unbounded::<oneshot::Sender<T>>();
        async_std::task::spawn(async move {
            let val = fut.await;
            while let Some(val_tx) = rx.next().await {
                let _ = val_tx.send(val.clone());
            }
        });
        AsyncOnce { sender: tx }
    }
    #[cfg(all(
        feature = "wasm-bindgen-futures",
        not(feature = "tokio"),
        not(feature = "async-std")
    ))]
    pub fn new<F>(fut: F) -> AsyncOnce<T>
    where
        F: Future<Output = T> + 'static,
    {
        let (tx, mut rx) = mpsc::unbounded::<oneshot::Sender<T>>();
        wasm_bindgen_futures::spawn_local(async move {
            let val = fut.await;
            while let Some(val_tx) = rx.next().await {
                let _ = val_tx.send(val.clone());
            }
        });
        AsyncOnce { sender: tx }
    }
    pub async fn get(&self) -> T {
        let (tx, rx) = oneshot::channel();
        self.sender.unbounded_send(tx).unwrap();
        rx.await.unwrap()
    }
}

#[cfg(all(
    feature = "tokio",
    not(feature = "async-std"),
    not(feature = "wasm-bindgen-futures")
))]
#[test]
fn lazy_static_test_for_tokio() {
    use lazy_static::lazy_static;
    use tokio::runtime::Runtime;
    lazy_static! {
        static ref FOO: AsyncOnce<u32> = AsyncOnce::new(async { 1 });
    }
    let mut rt = Runtime::new().unwrap();
    rt.block_on(async { assert_eq!(FOO.get().await, 1) })
}

#[cfg(all(
    feature = "async-std",
    not(feature = "tokio"),
    not(feature = "wasm-bindgen-futures")
))]
#[test]
fn lazy_static_test_for_async_std() {
    use async_std::task;
    use lazy_static::lazy_static;
    lazy_static! {
        static ref FOO: AsyncOnce<u32> = AsyncOnce::new(async { 1 });
    }
    task::block_on(async { assert_eq!(FOO.get().await, 1) })
}
