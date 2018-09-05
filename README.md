# rustricks

This readme is written to remember some tricks one can do in Rust to accomplish tasks where the solution is not obvious.

### Moving a mutable reference

Assigning a mutable slice to a subslice of itself is impossible without moving:

```Rust
fn main() {
    let mut array = [1, 2, 3, 4, 5, 6];

    let mut view = array.as_mut();

    view = &mut view[2..];

    println!("{:?}", view);
}
```

The borrowchecker complains that `view` is borrowed while assigning to `view`, which violates the borrowing rules.

To get around this, there are 2 options:
  - Use braces `{ }`.
  - Use a function that moves the borrow.

##### Braces technique

```Rust
fn main() {
    let mut array = [1, 2, 3, 4, 5, 6];

    let mut view = array.as_mut();

    view = &mut {view}[2..];

    println!("{:?}", view);
}
```

In this case, `view` is first moved into the braces, then indexed, than converted into a mutable slice, and thát is reassigned to `view`.

##### Moving the borrow

```Rust
fn mv<T>(x: T) -> T { x }

fn main() {
    let mut array = [1, 2, 3, 4, 5, 6];

    let mut view = array.as_mut();

    view = &mut mv(view)[2..];

    println!("{:?}", view);
}
```

In this case, `view` is moved into the `mv` function.