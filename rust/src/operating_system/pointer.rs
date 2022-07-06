use core::ops::{DerefMut, Deref};

use crate::interface::ApplicationFramework;

use super::OperatingSystem;

#[derive(PartialEq, Eq, Debug)]
pub struct OperatingSystemPointer<F: ApplicationFramework + 'static> {
    pub ptr: *mut OperatingSystem<F>,
}

impl<F: ApplicationFramework + 'static> Clone for OperatingSystemPointer<F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<F: ApplicationFramework + 'static> Copy for OperatingSystemPointer<F> {}

impl<F: ApplicationFramework + 'static> Deref for OperatingSystemPointer<F> {
    type Target = OperatingSystem<F>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref().unwrap() }
    }
}

impl<F: ApplicationFramework + 'static> DerefMut for OperatingSystemPointer<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut().unwrap() }
    }
}

impl<F: ApplicationFramework + 'static> OperatingSystemPointer<F> {
    pub fn get_mut_from_immut(&self) -> &mut OperatingSystem<F> {
        unsafe { self.ptr.as_mut().unwrap() }
    }

    pub fn new(ptr: *mut OperatingSystem<F>) -> Self {
        Self { ptr }
    }

    pub fn none() -> Self {
        Self::new(core::ptr::null_mut())
    }
}

pub trait OsAccessor<F: ApplicationFramework> {
    fn os(&self) -> &OperatingSystem<F>;

    #[allow(clippy::mut_from_ref)]
    fn os_mut(&self) -> &mut OperatingSystem<F>;
}

macro_rules! os_accessor {
    ($n:ty) => {
        #[allow(unused)]
        use crate::operating_system::OsAccessor as _;

        impl<F: ApplicationFramework> crate::operating_system::OsAccessor<F> for $n {
            #[allow(unused)]
            fn os(&self) -> &OperatingSystem<F> { core::ops::Deref::deref(&self.os) }

            #[allow(unused)]
            #[allow(clippy::mut_from_ref)]
            fn os_mut(&self) -> &mut OperatingSystem<F> { self.os.get_mut_from_immut() }        
        }
    };
}
pub(crate) use os_accessor;
