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
use std::ptr::null;
use std::sync::RwLock;
use std::task::Context;
use std::task::Poll;

type Fut<T> = RwLock<Result<T, Pin<Box<dyn Future<Output = T>>>>>;
pub struct AsyncOnce<T: 'static> {
    ptr: *const T,
    fut: Fut<T>,
}

unsafe impl<T: 'static> Sync for AsyncOnce<T> {}

impl<T> AsyncOnce<T> {
    pub fn new<F>(fut: F) -> AsyncOnce<T>
    where
        F: Future<Output = T> + 'static,
    {
        AsyncOnce {
            ptr: null(),
            fut: RwLock::new(Err(Box::pin(fut))),
        }
    }
    #[inline(always)]
    pub fn get(&'static self) -> &'static Self {
        self
    }
}

impl<T> Future for &'static AsyncOnce<T> {
    type Output = &'static T;
    #[inline(always)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<&'static T> {
        if let Some(ptr) = unsafe { self.ptr.as_ref() } {
            return Poll::Ready(ptr);
        }
        if let Ok(val) = self.fut.try_read() {
            if let Ok(val) = val.as_ref() {
                return Poll::Ready(unsafe { (val as *const T).as_ref().unwrap() });
            }
        }
        if let Ok(mut fut) = self.fut.try_write() {
            let mut result = None;
            if let Err(fut) = fut.as_mut() {
                if let Poll::Ready(val) = Pin::new(fut).poll(cx) {
                    result = Some(val);
                }
            }
            if let Some(val) = result {
                *fut = Ok(val);
                let ptr = fut.as_ref().ok().unwrap() as *const T;
                let this = (*self) as *const _ as *mut AsyncOnce<T>;
                unsafe {
                    (*this).ptr = ptr;
                }
                return Poll::Ready(unsafe { &*ptr });
            }
        }
        Poll::Pending
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
            Delay::new(Duration::from_millis(100)).await;
            1
        });
    }
    let rt = Builder::new_current_thread().build().unwrap();
    rt.spawn(async { assert_eq!(FOO.get().await, &1) });
    rt.spawn(async { assert_eq!(FOO.get().await, &1) });
    rt.spawn(async { assert_eq!(FOO.get().await, &1) });
    rt.block_on(async {
        Delay::new(Duration::from_millis(200)).await;
        assert_eq!(FOO.get().await, &1);
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
