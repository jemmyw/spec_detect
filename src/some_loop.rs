/**
Loop on future that returns an option. If the option is None thne break the
loop. Otherwise execute the block with the given bind.

```
some_loop!((event, app_state) = changed_files_stream.next() => {
    do something with event and app_state
})
```

expands to

```
loop {
    match changed_files_stream.next().await {
        Some((event, app_state)) => {
            do something with event and app_state
        }
        None => { break; }
    }
}
```
*/
#[macro_export]
macro_rules! some_loop {
    ($bind:pat = $fut:expr => $block:block) => {
        loop {
            let opt = $fut.await;

            match opt {
                None => {
                    break;
                }
                Some($bind) => $block,
            }
        }
    };
}
