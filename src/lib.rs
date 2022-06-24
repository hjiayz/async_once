//! ## async once tool for lazy_static
//!
//! # Examples
//! ```rust
//!    use lazy_static::lazy_static;
//!    use tokio::runtime::Builder;
//!    use async_once::AsyncOnce;
//!
//!    lazy_static!{
//!        static ref FOO : AsyncOnce<u32> = AsyncOnce::new(async{
//!            1
//!        });
//!    }
//!    let rt = Builder::new_current_thread().build().unwrap();
//!    rt.block_on(async {
//!        assert_eq!(FOO.get().await , &1)
//!    })
//! ```
//!
//! ### run tests
//!
//! ```bash
//!    cargo test
//!    wasm-pack test --headless --chrome --firefox
//! ```
//!

use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

pub struct AsyncOnce<T: 'static> {
    fut: parking_lot::Mutex<Pin<Box<dyn Future<Output = T> + Send + Sync>>>,
    value: once_cell::sync::OnceCell<T>,
}

impl<T> AsyncOnce<T> {
    pub fn new<F>(fut: F) -> AsyncOnce<T>
    where
        F: Future<Output = T> + Send + Sync + 'static,
    {
        Self {
            fut: parking_lot::Mutex::new(Box::pin(fut)),
            value: once_cell::sync::OnceCell::new(),
        }
    }
    #[inline(always)]
    pub fn get(&'static self) -> &'static Self {
        self
    }

    fn set_value(&'static self, value: T) -> &'static T {
        self
            .value
            .try_insert(value)
            .map_err(|_| ())
            .expect("The value was already set before")
    }
}

impl<T: Send + Sync + 'static> Future for &'static AsyncOnce<T> {
    type Output = &'static T;

    #[inline(always)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<&'static T> {
        if let Some(value) = self.value.get() {
            return Poll::Ready(value);
        }

        let mut fut = self.fut.lock();
        match Pin::new(&mut *fut).poll(cx) {
            Poll::Ready(value) => Poll::Ready((&**self).set_value(value)),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn lazy_static_test_for_tokio() {
    use futures_timer::Delay;
    use lazy_static::lazy_static;
    use std::time::Duration;
    use tokio::runtime::Builder;
    lazy_static! {
        static ref FOO: AsyncOnce<u32> = AsyncOnce::new(async {
            tokio::spawn(async { assert_eq!(FOO.get().await, &1) });
            Delay::new(Duration::from_millis(100)).await;
            1
        });
    }
    let rt = Builder::new_current_thread().build().unwrap();
    let handle1 = rt.spawn(async {
        Delay::new(Duration::from_millis(100)).await;
        assert_eq!(FOO.get().await, &1)
    });
    let handle2 = rt.spawn(async {
        Delay::new(Duration::from_millis(150)).await;
        assert_eq!(FOO.get().await, &1)
    });
    rt.block_on(async {
        use futures::future;
        Delay::new(Duration::from_millis(50)).await;
        let value = match future::select(FOO.get(), future::ready(1u32)).await {
            future::Either::Left((value, _)) => *value,
            future::Either::Right((value, _)) => value,
        };
        assert_eq!(&value, &1);
        let _ = handle1.await;
        let _ = handle2.await;
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn lazy_static_test_for_async_std() {
    use async_std::task;
    use futures_timer::Delay;
    use lazy_static::lazy_static;
    use std::time::Duration;
    lazy_static! {
        static ref FOO: AsyncOnce<u32> = AsyncOnce::new(async {
            Delay::new(Duration::from_millis(100)).await;
            1
        });
    }
    task::spawn(async { assert_eq!(FOO.get().await, &1) });
    task::spawn(async { assert_eq!(FOO.get().await, &1) });
    task::spawn(async { assert_eq!(FOO.get().await, &1) });
    task::block_on(async {
        Delay::new(Duration::from_millis(200)).await;
        assert_eq!(FOO.get().await, &1);
    });
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn lazy_static_test_for_smol() {
    use futures_timer::Delay;
    use lazy_static::lazy_static;
    use std::time::Duration;
    lazy_static! {
        static ref FOO: AsyncOnce<u32> = AsyncOnce::new(async {
            Delay::new(Duration::from_millis(100)).await;
            1
        });
    }
    smol::spawn(async { assert_eq!(FOO.get().await, &1) }).detach();
    smol::spawn(async { assert_eq!(FOO.get().await, &1) }).detach();
    smol::spawn(async { assert_eq!(FOO.get().await, &1) }).detach();
    smol::block_on(async {
        Delay::new(Duration::from_millis(200)).await;
        assert_eq!(FOO.get().await, &1);
    });
}
