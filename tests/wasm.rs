extern crate async_once;
extern crate lazy_static;
extern crate wasm_bindgen_test;
#[allow(unused_imports)]
use wasm_bindgen_test::*;

#[cfg(all(
    feature = "wasm-bindgen-futures",
    not(feature = "tokio"),
    not(feature = "async-std")
))]
#[wasm_bindgen_test]
async fn lazy_static_test_for_wasm() {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    lazy_static! {
        static ref FOO: AsyncOnce<u32> = AsyncOnce::new(async { 1 });
    }
    assert_eq!(FOO.get().await, 1)
}
