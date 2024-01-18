#![cfg_attr(not(test), no_std)]
mod api;
pub use api::*;
mod process;
use memory_addr::VirtAddr;
use flags::WaitStatus;
use link::raw_ptr_to_ref_str;
pub use process::{Process, PID2PC, TID2TASK};

use core::fmt::{Display, Debug};
use core::marker::PhantomData;

pub mod flags;
pub mod futex;
pub mod link;
pub mod loader;
mod stdio;

mod fd_manager;
#[cfg(feature = "signal")]
pub mod signal;

#[derive(Clone, Copy)]
pub struct UserRef<T> {
    addr: VirtAddr,
    r#type: PhantomData<T>,
}

impl<T> From<usize> for UserRef<T> {
    fn from(value: usize) -> Self {
        Self {
            addr: value.into(),
            r#type: PhantomData,
        }
    }
}

impl<T> From<VirtAddr> for UserRef<T> {
    fn from(value: VirtAddr) -> Self {
        Self {
            addr: value,
            r#type: PhantomData,
        }
    }
}

impl<T> Into<usize> for UserRef<T> {
    fn into(self) -> usize {
        self.addr.as_usize()
    }
}

impl<T> UserRef<T> {
    #[inline]
    pub fn get_mut_ptr(&self) -> *mut T {
        self.addr.as_usize() as *mut T
    }

    #[inline]
    pub fn get_ptr(&self) -> *const T {
        self.addr.as_usize() as *const T
    }

    #[inline]
    pub fn ptr_is_null(&self) -> bool {
        (self.addr.as_usize() as *const T).is_null()
    }

    #[inline]
    pub fn get_usize(&self) -> usize {
        self.addr.as_usize()
    }

    #[inline]
    pub fn get_mut_ref(&self) -> &'static mut T {
        unsafe {
            &mut *(self.addr.as_usize() as *mut T)
        }
    }

    #[inline]
    pub fn get_ref(&self) -> &'static T {
        unsafe {
            &*(self.addr.as_usize() as *const T)
        }
    }

    #[inline]
    pub fn slice_mut_with_len(&self, len: usize) -> &'static mut [T] {
        unsafe { 
            core::slice::from_raw_parts_mut(self.get_mut_ptr(), len) 
        }
    }

    #[inline]
    pub fn slice_mut_with_unmut_len(&self, len: usize) -> &'static [T] {
        unsafe {
            core::slice::from_raw_parts(self.get_mut_ptr(), len) 
        }
    }

    #[inline]
    pub fn add_count(&self, count: usize) -> *mut T {
        unsafe{self.get_mut_ptr().add(count)}
    }

    #[inline]
    pub fn copy_nonoverlapping(&self, cwd: &[T], len: usize) {
        let dst = self.get_mut_ptr() as *mut T;
        unsafe {
            core::ptr::copy_nonoverlapping(cwd.as_ptr(), dst, len);
        }
    }

    #[inline]
    pub fn raw_ptr_to_ref_str(&self) -> &str {
        unsafe {
            raw_ptr_to_ref_str(self.get_mut_ptr() as *const u8)
        }
    }

    #[inline]
    pub fn wait_pid(&self, pid: isize) -> Result<u64, WaitStatus> {
        let exit_code_ptr = self.get_mut_ptr() as *mut i32;
        unsafe { wait_pid(pid, exit_code_ptr) }
    }

    #[inline]
    pub fn get_t(&self, count: usize) -> &'static T {
        unsafe {
            &*(self.add_count(count))
        }
    }

    #[inline]
    pub fn write_offset(&self, offset: isize, fd_num: T) {
        unsafe {
            core::ptr::write(self.get_mut_ptr().offset(offset), fd_num);
        }
    }

    #[inline]
    pub fn get_adjacent_mut_ref(&self, stime: *mut T) -> &mut T {
        unsafe {
            &mut *stime
        }
    }
}

impl<T> Display for UserRef<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "{}({:#x})",
            core::any::type_name::<T>(),
            self.addr.as_usize()
        ))
    }
}

impl<T> Debug for UserRef<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "{}({:#x})",
            core::any::type_name::<T>(),
            self.addr.as_usize()
        ))
    }
}

pub fn read_u32_from_addr(addr: VirtAddr) -> u32 {
    // 安全地读取数据
    let value = unsafe { (addr.as_usize() as *const u32).read_volatile() };
    value
}

pub fn read_from_addr<U>(addr: usize) -> &'static U {
    let ptr = addr as *const U;
    let data = unsafe {
        &*ptr
    };
    data
}
