#![cfg_attr(all(not(test), not(doc)), no_std)]
use axprocess::current_process;
use axprocess::{flags::WaitStatus, link::raw_ptr_to_ref_str, wait_pid};
use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use memory_addr::VirtAddr;
pub type SyscallError = axerrno::LinuxError;

#[derive(Clone, Copy)]
pub struct UserRef<T> {
    // checked: bool,
    addr: VirtAddr,
    r#type: PhantomData<T>,
}

#[derive(Clone, Copy)]
pub struct UserRefSlice<T> {
    useref: UserRef<T>,
    len: usize,
}

pub enum CheckType {
    Lazy,
    TypeLazy,
    RangeLazy(VirtAddr),
}

impl<T> Into<UserRef<T>> for UserRefSlice<T> {
    fn into(self) -> UserRef<T> {
        self.useref
    }
}

impl<T> From<usize> for UserRef<T> {
    fn from(value: usize) -> Self {
        Self {
            // checked: false,
            addr: value.into(),
            r#type: PhantomData,
        }
    }
}

impl<T> From<VirtAddr> for UserRef<T> {
    fn from(value: VirtAddr) -> Self {
        Self {
            // checked: false,
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

impl<T> From<(UserRef<T>, usize)> for UserRefSlice<T> {
    fn from(path: (UserRef<T>, usize)) -> Self {
        Self {
            useref: path.0,
            len: path.1,
        }
    }
}

impl<T> UserRef<T> {
    #[inline]
    pub fn get_mut_ptr(&self, check_type: CheckType) -> Result<*mut T, SyscallError> {
        let is_ok = match check_type {
            CheckType::Lazy => self.manual_alloc_for_lazy_is_ok(),
            CheckType::TypeLazy => self.manual_alloc_type_for_lazy_is_ok(),
            CheckType::RangeLazy(end) => self.manual_alloc_range_for_lazy_is_ok(end),
        };
    
        if is_ok {
            Ok(self.addr.as_usize() as *mut T)
        } else {
            Err(SyscallError::EFAULT)
        }
    }

    #[inline]
    pub fn get_ptr(&self, check_type: CheckType) -> Result<*const T, SyscallError> {
        let is_ok = match check_type {
            CheckType::Lazy => self.manual_alloc_for_lazy_is_ok(),
            CheckType::TypeLazy => self.manual_alloc_type_for_lazy_is_ok(),
            CheckType::RangeLazy(end) => self.manual_alloc_range_for_lazy_is_ok(end),
        };

        if is_ok {
            Ok(self.addr.as_usize() as *const T)
        } else {
            Err(SyscallError::EFAULT)
        }
    }

    #[inline]
    pub fn get_mut_ref(&self, check_type: CheckType) -> Result<&'static mut T, SyscallError> {
        let is_ok = match check_type {
            CheckType::Lazy => self.manual_alloc_for_lazy_is_ok(),
            CheckType::TypeLazy => self.manual_alloc_type_for_lazy_is_ok(),
            CheckType::RangeLazy(end) => self.manual_alloc_range_for_lazy_is_ok(end),
        };

        if is_ok {
            Ok(unsafe { &mut *(self.addr.as_usize() as *mut T) })
        } else {
            Err(SyscallError::EFAULT)
        }
    }

    #[inline]
    pub fn get_ref(&self, check_type: CheckType) -> Result<&'static T, SyscallError> {
        let is_ok = match check_type {
            CheckType::Lazy => self.manual_alloc_for_lazy_is_ok(),
            CheckType::TypeLazy => self.manual_alloc_type_for_lazy_is_ok(),
            CheckType::RangeLazy(end) => self.manual_alloc_range_for_lazy_is_ok(end),
        };

        if is_ok {
            Ok(unsafe { &*(self.addr.as_usize() as *const T) })
        } else {
            Err(SyscallError::EFAULT)
        }
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        (self.addr.as_usize() as *const T).is_null()
    }

    #[inline]
    pub fn get_usize(&self) -> usize {
        self.addr.as_usize()
    }

    #[inline]
    pub fn add(&self, count: usize) -> *mut T {
        unsafe { self.get_mut_ptr(CheckType::Lazy).unwrap().add(count) }
    }

    #[inline]
    pub fn copy_nonoverlapping(&self, cwd: &[T], len: usize) {
        let dst = self.get_mut_ptr(CheckType::Lazy).unwrap() as *mut T;
        unsafe {
            core::ptr::copy_nonoverlapping(cwd.as_ptr(), dst, len);
        }
    }

    #[inline]
    pub fn raw_ptr_to_ref_str(&mut self, check_type: CheckType) -> Result<&str, SyscallError> {
        let is_ok = match check_type {
            CheckType::Lazy => self.manual_alloc_for_lazy_is_ok(),
            CheckType::TypeLazy => self.manual_alloc_type_for_lazy_is_ok(),
            CheckType::RangeLazy(end) => self.manual_alloc_range_for_lazy_is_ok(end),
        };

        if is_ok {
            // self.checked = true;
            Ok(unsafe { raw_ptr_to_ref_str(self.get_mut_ptr(CheckType::Lazy).unwrap() as *const u8)})      
        } else {
            Err(SyscallError::EFAULT)
        }
    }

    #[inline]
    pub fn wait_pid(&self, pid: isize) -> Result<u64, WaitStatus> {
        let exit_code_ptr = self.get_mut_ptr(CheckType::Lazy).unwrap() as *mut i32;
        unsafe { wait_pid(pid, exit_code_ptr) }
    }

    #[inline]
    pub fn get_t(&self, count: usize, check_type: CheckType) -> Result<&'static T, SyscallError> {
        let is_ok = match check_type {
            CheckType::Lazy => self.manual_alloc_for_lazy_is_ok(),
            CheckType::TypeLazy => self.manual_alloc_type_for_lazy_is_ok(),
            CheckType::RangeLazy(end) => self.manual_alloc_range_for_lazy_is_ok(end),
        };

        if is_ok {
            Ok(unsafe { &*(self.add(count)) })
        } else {
            Err(SyscallError::EFAULT)
        }   
    }

    #[inline]
    pub fn write_offset(&self, offset: isize, fd_num: T, check_type: CheckType) -> Result<(), SyscallError> {
        let is_ok = match check_type {
            CheckType::Lazy => self.manual_alloc_for_lazy_is_ok(),
            CheckType::TypeLazy => self.manual_alloc_type_for_lazy_is_ok(),
            CheckType::RangeLazy(end) => self.manual_alloc_range_for_lazy_is_ok(end),
        };

        if is_ok {
            unsafe {
                core::ptr::write(self.get_mut_ptr(CheckType::Lazy).unwrap().offset(offset), fd_num);
            }
            Ok(())
        } else {
            Err(SyscallError::EFAULT)
        }
    }

    #[inline]
    pub fn get_adjacent_mut_ref(&self, stime: *mut T) -> &mut T {
        unsafe { &mut *stime }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.addr.as_usize() != 0
    }

    #[inline]
    pub fn manual_alloc_for_lazy_is_ok(&self) -> bool {
        // if self.checked {
        //     return true;
        // }
        // let process = current_process(); // 仅获取一次当前进程
        // if process.manual_alloc_for_lazy(self.get_usize().into()).is_ok() {
        //     self.checked = true; // 更新状态
        //     true
        // } else {
        //     false
        // } 
        // Err(SyscallError::EINVAL)
        let process = current_process(); // 仅获取一次当前进程
        process.manual_alloc_for_lazy(self.get_usize().into()).is_ok()
    }

    #[inline]
    pub fn manual_alloc_type_for_lazy_is_ok(&self) -> bool{
        // if self.checked {
        //     return true;
        // }
        // let process = current_process(); // 仅获取一次当前进程
        // if process.manual_alloc_type_for_lazy(self.get_ptr().unwrap()).is_ok() {
        //     self.checked = true; // 更新状态
        //     true
        // } else {
        //     false
        // }
        // Err(SyscallError::EFAULT)
        let process = current_process(); // 仅获取一次当前进程
        process.manual_alloc_type_for_lazy(self.get_ptr(CheckType::Lazy).unwrap()).is_ok()
    }

    #[inline]
    pub fn manual_alloc_range_for_lazy_is_ok(&self, end: VirtAddr) -> bool {
        // if self.checked {
        //     return true;
        // }
        // let process = current_process(); // 仅获取一次当前进程
        // if process.manual_alloc_range_for_lazy(self.get_usize().into(), end).is_ok() {
        //     self.checked = true; // 更新状态
        //     true
        // } else {
        //     false
        // }
        // Err(SyscallError::EFAULT)
        let process = current_process(); // 仅获取一次当前进程
        process.manual_alloc_range_for_lazy(self.get_usize().into(), end).is_ok()
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

impl<T> UserRefSlice<T> {
    #[inline]
    pub fn useref(&self) -> &UserRef<T> { 
        &self.useref
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn from_raw_parts_mut(&self, check_type: CheckType) -> Result<&'static mut [T], SyscallError> {
        let is_ok = match check_type {
            CheckType::Lazy => self.useref.manual_alloc_for_lazy_is_ok(),
            CheckType::TypeLazy => self.useref.manual_alloc_type_for_lazy_is_ok(),
            CheckType::RangeLazy(end) => self.useref.manual_alloc_range_for_lazy_is_ok(end),
        };

        if is_ok {
            Ok(unsafe { core::slice::from_raw_parts_mut(self.useref.get_mut_ptr(CheckType::Lazy).unwrap(), self.len) })
        } else {
            Err(SyscallError::EFAULT)
        }   
    }

    #[inline]
    pub fn from_raw_parts(&self, check_type: CheckType) -> Result<&'static [T], SyscallError> {
        let is_ok = match check_type {
            CheckType::Lazy => self.useref.manual_alloc_for_lazy_is_ok(),
            CheckType::TypeLazy => self.useref.manual_alloc_type_for_lazy_is_ok(),
            CheckType::RangeLazy(end) => self.useref.manual_alloc_range_for_lazy_is_ok(end),
        };

        if is_ok {
            Ok(unsafe { core::slice::from_raw_parts(self.useref.get_ptr(CheckType::Lazy).unwrap(), self.len) })      
        } else {
            Err(SyscallError::EFAULT)
        }
    }

    #[inline]
    pub fn copy_nonoverlapping(&self, cwd: &[T], len: usize, check_type: CheckType) -> Result<(), SyscallError>{
        let is_ok = match check_type {
            CheckType::Lazy => self.useref.manual_alloc_for_lazy_is_ok(),
            CheckType::TypeLazy => self.useref.manual_alloc_type_for_lazy_is_ok(),
            CheckType::RangeLazy(end) => self.useref.manual_alloc_range_for_lazy_is_ok(end),
        };

        if is_ok {
            let dst = self.useref().get_mut_ptr(CheckType::Lazy).unwrap() as *mut T;
            unsafe {
                core::ptr::copy_nonoverlapping(cwd.as_ptr(), dst, len);
            }
            Ok(())
        } else {
            Err(SyscallError::EFAULT)
        }
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        (self.useref.addr.as_usize() as *const T).is_null()
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.len != 0
    }

    #[inline]
    pub fn manual_alloc_range_for_lazy_is_ok(&self, end: VirtAddr) -> bool {
        // if self.checked {
        //     return true;
        // }
        // let process = current_process(); // 仅获取一次当前进程
        // if process.manual_alloc_range_for_lazy(self.get_usize().into(), end).is_ok() {
        //     self.checked = true; // 更新状态
        //     true
        // } else {
        //     false
        // }
        // Err(SyscallError::EFAULT)
        let process = current_process(); // 仅获取一次当前进程
        process.manual_alloc_range_for_lazy(self.useref().get_usize().into(), end).is_ok()
    }
}

pub fn read_u32_from_addr(addr: VirtAddr) -> u32 {
    // 安全地读取数据
    let value = unsafe { (addr.as_usize() as *const u32).read_volatile() };
    value
}

pub fn read_from_addr<U>(addr: usize) -> &'static U {
    let ptr = addr as *const U;
    let data = unsafe { &*ptr };
    data
}
