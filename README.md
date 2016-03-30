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
todotxt = "^0.1"
```

Then use it:

```rust
extern crate todotxt;

use todotxt::TodoItem;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let mut todo_file = BufReader::new(File::open("/home/kstep/todo/todo.txt").unwrap());
    for line in todo_file.lines() {
        let todo_item: TodoItem = line.unwrap().parse().unwrap();
        // Now work with TodoItem
    }
}
```

[1]: http://todotxt.com/
[2]: https://github.com/ginatrapani
