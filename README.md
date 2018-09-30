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
  - Rewrite and leverage NLL

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

#### Technique 3: Rewrite and leverage NLL

```Rust
#![feature(nll)]

#[derive(Debug)]
struct NonCopyBool(bool);

fn main() {
    let mut flag = NonCopyBool(false);

    let mut c1 = || flag = NonCopyBool(true);
    c1();

    let c2 = || println!("{:?}", flag);
    c2();
}
```
[playground](https://play.rust-lang.org/?gist=d4bdc496c789939d41a6ad9cdbb33b5a&version=nightly&mode=debug&edition=2015)

##### How does it work?
In this case, at no point there are both a mutable reference and a reference to the flag, and therfore there is no problem in the first place.

##### Advantages
The advantage of this technique is that is does not require any runtime checks, and does not require any implementation of traits.

##### Disadvantages
This technique can not always be used, and is not verry flexible.

### Being generic over [T; N]

It occasionally happens you want to write a function that is generic over array's, and you might write something like:

```Rust
fn example<T, A>(xs: A)
where
    A: AsRef<[T]>
{
    // Do something
}

fn main() {
    let array = [0; 10];

    example(array);
}
```

However, when the array-size is 33 or more, AsRef is not implemented anymore:
```
error[E0277]: the trait bound `[{integer}; 33]: std::convert::AsRef<[_]>` is not satisfied
  --> src/main.rs:11:5
   |
11 |     example(array);
   |     ^^^^^^^ the trait `std::convert::AsRef<[_]>` is not implemented for `[{integer}; 33]`
```

In stable Rust there is no solution, however in nightly there is.

#### Unsize

```Rust
#![feature(unsize)]

use std::marker::Unsize;

fn example<T, A>(xs: A)
where
    A: Unsize<[T]>
{
    // Do something
}

fn main() {
    let array = [0; 33];

    example(array);
}
```

Now the example function takes anything that can be `Unsized` into a [T]. It happens to be that any array can be unsized into a [T], because any array implements `Unsize<[T]>`. The compiler automatically implements it!

##### Advantages
The advantage is that with `Unsize<[T]>`, you can pass any array into the function, the size of the array does not matter anymore.

##### Disadvantages
The disadvantage is that now you can *only* pass in array's. Slices, Boxes, Vec's, e.g can't be passed into the function anymore.

### References to references.

Somethimes, you want to have a reference to a reference, e.g
```Rust
&'a &'b T
```

This requires `'b: 'a`, NOT `'a: 'b`

It is no different than
```Rust
&'a T
```
which requires `T: 'a`
