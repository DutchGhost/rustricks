/// Forces a move.
pub fn mv<T>(x: T) -> T { x }


use std::cell::UnsafeCell;
use std::marker::PhantomData;

#[derive(Debug)]
struct NonCopyBool(bool);

struct UnsafeShared<T> {
    inner: UnsafeCell<T>,
}

struct UnsafeRef<'a, T: 'a> {
    ptr: *mut T,
    marker: PhantomData<&'a mut T>
}

impl <'a, T: 'a> UnsafeRef<'a, T> {
    fn new(ptr: *mut T) -> Self {
        Self {ptr, marker: PhantomData}
    }
}
impl <'a, T>std::ops::Deref for UnsafeRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { & *self.ptr }
    }
}

impl <'a, T>std::ops::DerefMut for UnsafeRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl <'a, T: 'a> Drop for UnsafeRef<'a, T> {
    fn drop(&mut self) {
        println!("{:p}", self);
    }
}

impl<T> UnsafeShared<T> {
    fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }

    /// The caller must ensure there are no references to the inner value when this is called.
    fn borrow_mut(&self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }

    /// The caller must ensure there is no mutable reference to the inner value when this is called.
    fn borrow(&self) -> &T {
        unsafe { &*self.inner.get() }
    }

    fn as_ref<'a>(&self) -> UnsafeRef<'a, T> {
        UnsafeRef::new(self.inner.get())
    }
}

fn main() {
    let flag = UnsafeShared::new(NonCopyBool(false));

    let c1 = || *flag.as_ref() = NonCopyBool(true);
    let c2 = || println!("{:?}", *flag.as_ref());

    c1();
    c2();
}
