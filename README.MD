## async once tool for lazy_static

### features:
* tokio
* async-std
* wasm-bindgen-futures

### sample 
```rust
    use lazy_static::lazy_static;
    use tokio::runtime::Runtime;
    // tokio feature
    use async_once::AsyncOnce;

    lazy_static!{
        static ref FOO : AsyncOnce<u32> = AsyncOnce::new(async{
            1
        });
    }
    let mut rt = Runtime::new().unwrap();
    rt.block_on(async {
        assert_eq!(FOO.get().await , 1)
    })
```

### run tests

```
    cargo test --features tokio
    cargo test --features async-std
    wasm-pack test --node -- --features wasm-bindgen-futures
```