use std::ops::{Deref, DerefMut};

/// An owned or mutably borrowed type
pub enum Owm<'a, T> {
    Owned(T),
    Borrowed(&'a mut T),
}

impl<'a, T> Deref for Owm<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Owm::Owned(v) => v,
            Owm::Borrowed(v) => v,
        }
    }
}

impl<'a, T> DerefMut for Owm<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Owm::Owned(v) => v,
            Owm::Borrowed(v) => v,
        }
    }
}

pub enum Rop<'a, T> {
    Owned(*const T),
    Borrowed(&'a T),
}

impl<'a, T> Rop<'a, T> {
    pub fn from_ref(r: &'a T) -> Self {
        Self::Borrowed(r)
    }

    pub fn from_owned(v: T) -> Self {
        Self::Owned(Box::into_raw(Box::new(v)))
    }

    pub fn get(&self) -> &'a T {
        match self {
            // SAFETY: Generally not
            Rop::Owned(p) => unsafe { p.as_ref().expect("Should be valid.") },
            Rop::Borrowed(v) => v,
        }
    }
}

impl<'a, T> Drop for Rop<'a, T> {
    fn drop(&mut self) {
        match self {
            Rop::Owned(ptr) => {
                let ptr = *ptr;
                // SAFETY: Generally not
                drop(unsafe { Box::from_raw(ptr as *mut T) })
            }
            Rop::Borrowed(_) => (),
        }
    }
}
