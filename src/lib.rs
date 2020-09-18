//! ## async once tool for lazy_static
//! 
//! # Examples
//! ```rust
//!    use lazy_static::lazy_static;
//!    use tokio::runtime::Runtime;
//!    use async_once::AsyncOnce;
//!
//!    lazy_static!{
//!        static ref FOO : AsyncOnce<u32> = AsyncOnce::new(async{
//!            1
//!        });
//!    }
//!    let mut rt = Runtime::new().unwrap();
//!    rt.block_on(async {
//!        assert_eq!(FOO.get().await , &1)
//!    })
//! ```
//! 
//! ### run tests
//! 
//! ```
//! cargo test
//! wasm-pack test --node
//! ```

use futures::future::FutureExt;
#[allow(unused_imports)]
use std::future::Future;
use std::pin::Pin;
use std::sync::RwLock;
use std::task::Context;
use std::task::Poll;
pub struct AsyncOnce<T> {
    fut: RwLock<Result<T, Pin<Box<dyn Future<Output = T> + Sync + Send>>>>,
}

impl<T> AsyncOnce<T> {
    pub fn new<F>(fut: F) -> AsyncOnce<T>
    where
        F: Future<Output = T> + 'static + Sync + Send,
    {
        let fut: RwLock<Result<T, Pin<Box<dyn Future<Output = T> + Sync + Send>>>> =
            RwLock::new(Err(Box::pin(fut)));
        AsyncOnce { fut }
    }
    pub async fn get(&'static self) -> &'static T {
        self.await
    }
}

impl<T> Future for &'static AsyncOnce<T>
where
    T: 'static,
{
    type Output = &'static T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<&'static T> {
        loop {
            if let Ok(val) = self.fut.try_read() {
                if let Ok(val) = val.as_ref() {
                    return Poll::Ready(unsafe { (val as *const T).as_ref().unwrap() });
                }
            }
            if let Ok(mut fut) = self.fut.try_write() {
                let mut result = None;
                if let Err(fut) = fut.as_mut() {
                    if let Poll::Ready(val) = fut.poll_unpin(cx) {
                        result = Some(val);
                    }
                }
                if result.is_some() {
                    *fut = Ok(result.unwrap());
                    continue;
                }
            }
            break;
        }
        Poll::Pending
    }
}

#[test]
fn lazy_static_test_for_tokio() {
    use lazy_static::lazy_static;
    use tokio::runtime::Runtime;
    lazy_static! {
        static ref FOO: AsyncOnce<u32> = AsyncOnce::new(async { 1 });
    }
    let mut rt = Runtime::new().unwrap();
    rt.block_on(async { assert_eq!(FOO.get().await, &1) })
}

#[test]
fn lazy_static_test_for_async_std() {
    use async_std::task;
    use lazy_static::lazy_static;
    lazy_static! {
        static ref FOO: AsyncOnce<u32> = AsyncOnce::new(async { 1 });
    }
    task::block_on(async { assert_eq!(FOO.get().await, &1) })
}
