extern crate async_once;
extern crate lazy_static;
extern crate wasm_bindgen_test;

#[allow(unused_imports)]
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn lazy_static_test_for_wasm() {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use wasm_bindgen_futures::spawn_local;

    lazy_static! {
        static ref FOO: AsyncOnce<u32> = AsyncOnce::new(async {
            1
        });
    }

    spawn_local(async { assert_eq!(FOO.get().await, &1) });
    spawn_local(async { assert_eq!(FOO.get().await, &1) });
    spawn_local(async { assert_eq!(FOO.get().await, &1) });
    assert_eq!(FOO.get().await, &1)
}
