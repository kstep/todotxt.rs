# [Todo.txt][1] file format parser

This is a **todo.txt** file format parser. The format was proposed by [Gina Trapani][2].
This parser supports some custom tags:

* Due date (`due:YYYY-MM-DD`).
* Threshold date (`t:YYYY-MM-DD`).
* Recurrent tasks (`rec:+?[0-9]+[dbmy]`).

It also parses all @contexts and +projects. Also, it additionally parses #hashtags.

Usage is very simple. First add it to your `Cargo.toml`:

```toml
[dependencies]
todotxt = "^0.3"
```

Then use it:

```rust
extern crate todotxt;

use todotxt::Task;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let mut todo_file = BufReader::new(File::open("/home/kstep/todo/todo.txt").unwrap());
    for line in todo_file.lines() {
        let todo_item: Task = line.unwrap().parse().unwrap();
        // Now work with Task
    }
}
```

[1]: http://todotxt.com/
[2]: https://github.com/ginatrapani

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
