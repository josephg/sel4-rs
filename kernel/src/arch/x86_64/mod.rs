use core::ffi::{c_char, CStr};
use core::marker::PhantomData;
use ufmt::derive::uDebug;

pub mod constants;
pub mod hardware;
mod boot;
mod acpi;
mod machine;
mod cpu;

#[cfg(feature = "smp")]
mod smp;
mod pic;
mod asm;
mod interrupt;
pub mod devices;

/// This is a wrapper for u32 values we read from system descriptor tables which are actually
/// pointers to some data.
// I'd love to just use #[derive(Copy, Clone)] here but those impls would be conditional on
// T: Copy, Clone, and that would be wrong.
#[repr(transparent)]
pub(crate) struct U32Ptr<T>(pub u32, PhantomData<T>);

impl<T> Clone for U32Ptr<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}
impl<T> Copy for U32Ptr<T> {}

impl<T> U32Ptr<T> {
    pub fn new(addr: u32) -> Self {
        Self(addr, PhantomData)
    }

    pub fn not_null(&self) -> bool {
        self.0 != 0
    }

    /// Get the raw pointer out. Note this is safe. Its only potentially unsafe to dereference the
    /// pointer.
    pub fn as_ptr(&self) -> *const T {
        // Extra coersion probably unnecessary.
        self.0 as usize as *const T
    }

    pub unsafe fn as_static_ref(self) -> &'static T {
        unsafe { &*self.as_ptr() }
    }
}

/// This is a wrapper for 32 bit pointers to C strings provided by multiboot and others.
#[derive(uDebug, Copy, Clone)]
#[repr(transparent)]
pub(crate) struct CStr32(u32);

impl CStr32 {
    /// Get the pointer as a pointer to a c_char.
    pub fn as_ptr(self) -> *const c_char {
        self.0 as usize as *const c_char
    }

    /// SAFETY:
    ///
    /// The string must be valid, and in valid memory.
    ///
    /// This function takes a container object as a parameter. The lifetime of the container object
    /// is used as the lifetime of the returned cstr.
    pub unsafe fn try_as_cstr<P>(self, _container: &P) -> Option<&CStr> {
        unsafe {
            // If the cstr is null, its not clear what the behaviour should be. In classic C spec
            // fashion, there is no documentation on whether or not any given string is guaranteed
            // to exist.
            //
            // We could interpret null strings as empty, but thats not quite right. Or we could
            // panic - which is reasonable behaviour.
            if self.0 == 0 {
                None
            } else {
                Some(CStr::from_ptr(self.as_ptr()))
            }
        }
    }
}

