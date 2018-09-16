# rustricks

This readme is written to remember some tricks one can do in Rust to accomplish tasks where the solution is not obvious.

### Moving a mutable reference

Lets say you want to assign a mutable slice to a subslice of itself. It currently is impossible to do such a thing 'directly':
```Rust
fn main() {
    let mut array = [1, 2, 3, 4, 5, 6];

    let mut view = array.as_mut();

    view = &mut view[2..];

    println!("{:?}", view);
}
```
[playground](https://play.rust-lang.org/?gist=9cf363fb0dc40511f5946d0670d48334&version=stable&mode=debug&edition=2015)

#### Why doesn't this work?
The borrowchecker complains that `view` is borrowed while assigning to `view`, which violates the borrowing rules.

To get around this, there are 3 options:
  - Use braces `{ }`.
  - Use a function that moves the borrow.
  - Use Non Lexical Lifetimes (NLL)

#### Technique 1: Use braces

```Rust
fn main() {
    let mut array = [1, 2, 3, 4, 5, 6];

    let mut view = array.as_mut();

    view = &mut {view}[2..];

    println!("{:?}", view);
}
```
[playground](https://play.rust-lang.org/?gist=a55056182ceafec5a7dc72cb5d16ff49&version=stable&mode=debug&edition=2015)

In this case, `view` is first moved into the braces, then indexed, than converted into a mutable slice, and thàt is reassigned to `view`.

#### Technique 2: Moving the borrow

```Rust
fn mv<T>(x: T) -> T { x }

fn main() {
    let mut array = [1, 2, 3, 4, 5, 6];

    let mut view = array.as_mut();

    view = &mut mv(view)[2..];

    println!("{:?}", view);
}
```
[playground](https://play.rust-lang.org/?gist=31c352ec759529cbf649319552ee1208&version=stable&mode=debug&edition=2015)

In this case, `view` is moved into the `mv` function, then indexed, than converted into a mutable slice, and then `view` is reassigned.

#### Technique 3: Use NLL

```Rust
#![feature(nll)]

fn main() {
    let mut array = [1, 2, 3, 4, 5, 6];

    let mut view = array.as_mut();

    view = &mut view[2..];

    println!("{:?}", view);
}
```
[playground](https://play.rust-lang.org/?gist=b890fefba2dc07ec5b12f112cc53cf29&version=nightly&mode=debug&edition=2015)

### Sharing a value with multiple closures

Whenever you create 2 closures that both capture the same variable, you'll get a compiler error:

```Rust
#[derive(Debug)]
struct NonCopyBool(bool);

fn main() {
    let mut flag = NonCopyBool(false);
    
    let mut c1 = || flag = NonCopyBool(true);
    let c2 = || println!("{:?}", flag);

    c1();
    c2();
}
```

```
rror[E0502]: cannot borrow `flag` as immutable because it is also borrowed as mutable
  --> src/main.rs:8:14
   |
7  |     let mut c1 = || flag = NonCopyBool(true);
   |                  -- ---- previous borrow occurs due to use of `flag` in closure
   |                  |
   |                  mutable borrow occurs here
8  |     let c2 = || println!("{:?}", flag);
   |              ^^                  ---- borrow occurs due to use of `flag` in closure
   |              |
   |              immutable borrow occurs here
...
12 | }
   | - mutable borrow ends here
```
[playground](https://play.rust-lang.org/?gist=787545229aa111a8e86f9692fb757928&version=stable&mode=debug&edition=2015)

#### Why doesn't this work?
The flag is borrowed mutably in the first closure, but also borrowed by reference in the second closure.
This means there are a mutable, and a non-mutable reference to the same data, which violates the rules.

To get around this, there are a few techniques:
    - Use [std::cell::Cell](https://doc.rust-lang.org/std/cell/struct.Cell.html).
    - Use [std::cell::RefCell](https://doc.rust-lang.org/std/cell/struct.RefCell.html).

#### Technique 1: std::cell::Cell

```Rust
use std::cell::Cell;

#[derive(Debug, Default)]
struct NonCopyBool(bool);

fn main() {
    let flag = Cell::new(NonCopyBool(false));

    let c1 = || flag.set(NonCopyBool(true));
    let c2 = || println!("{:?}", flag.take());

    c1();
    c2();
}
```
[playground](https://play.rust-lang.org/?gist=4f8610d164e2dacfdb1588c598b7cf8c&version=stable&mode=debug&edition=2015)

##### How does it work?
This technique works, because the [set](https://doc.rust-lang.org/std/cell/struct.Cell.html#method.set) method called in in the first closure, only takes the cell by reference.
In the second closure, the [take](https://doc.rust-lang.org/std/cell/struct.Cell.html#method.take) method is called on the cell. [take](https://doc.rust-lang.org/std/cell/struct.Cell.html#method.take) also takes the cell by reference, but requiress the inner value to implement [Default](https://doc.rust-lang.org/std/default/trait.Default.html). `Take` replaces the current value with the default value of the type it's holding, and returns the replaced value.

Over all, the flag is never mutably borrowed here, so it compiles.

##### Advantages
Advantages of this technique is that the inner value of the cell is not required to implement [Copy](https://doc.rust-lang.org/std/marker/trait.Copy.html), and does not have any runtime checks.

##### Disadvantages
The disadvantage of this technique is that the inner value is required to implement [Default](https://doc.rust-lang.org/std/default/trait.Default.html).

#### Technique 2: std::cell::RefCell

```Rust
use std::cell::RefCell;

#[derive(Debug)]
struct NonCopyBool(bool);

fn main() {
    let flag = RefCell::new(NonCopyBool(false));

    let c1 = || *flag.borrow_mut() = NonCopyBool(true);
    let c2 = || println!("{:?}", flag.borrow());

    c1();
    c2();
}
```
[playground](https://play.rust-lang.org/?gist=fcaec5b52590771cc6cd295ab7db40cd&version=stable&mode=debug&edition=2015)

##### How does it work?

This technique is also valid, because RefCell's [borrow_mut](https://doc.rust-lang.org/std/cell/struct.RefCell.html#method.borrow_mut) method called in the first closure, takes the cell by reference. It retruns a [RefMut](https://doc.rust-lang.org/std/cell/struct.RefMut.html) struct, which implements [Deref](https://doc.rust-lang.org/std/ops/trait.Deref.html) and [DerefMut](https://doc.rust-lang.org/std/ops/trait.DerefMut.html).
The second closures calls [borrow](https://doc.rust-lang.org/std/cell/struct.RefCell.html#method.borrow), which returns a [Ref](https://doc.rust-lang.org/std/cell/struct.Ref.html) struct.

##### Advantages
The advantages of this technique is that the inner value is not required to implement [Copy](https://doc.rust-lang.org/std/marker/trait.Copy.html) or [Default](https://doc.rust-lang.org/std/default/trait.Default.html). 

##### Disadvantages
The big disadvantage of RefCell is that obtaining references / mutable references is dynamically checked, and therefore has some runtime overhead.