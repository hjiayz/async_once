# async_once

### async once tool for lazy_static

## Examples
```rust
   use lazy_static::lazy_static;
   use tokio::runtime::Runtime;
   use async_once::AsyncOnce;

   lazy_static!{
       static ref FOO : AsyncOnce<u32> = AsyncOnce::new(async{
           1
       });
   }
   let mut rt = Runtime::new().unwrap();
   rt.block_on(async {
       assert_eq!(FOO.get().await , &1)
   })
```

#### run tests

cargo test
wasm-pack test --headless --chrome --firefox


License: MIT OR Apache-2.0
