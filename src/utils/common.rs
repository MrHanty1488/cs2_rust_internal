#[macro_export]
macro_rules! cast {
    ($ptr:expr, $ty:ty) => {
        unsafe { core::mem::transmute::<_, $ty>($ptr) }
    };
    ($ptr:expr) => {
        unsafe { core::mem::transmute($ptr) }
    };
}

#[inline(always)]
pub fn ptr_add<T>(ptr: *const T, offset: usize) -> *const T {
    unsafe { (ptr as *const u8).add(offset) as *const T }
}

#[inline(always)]
pub fn ptr_add_mut<T>(ptr: *mut T, offset: usize) -> *mut T {
    unsafe { (ptr as *mut u8).add(offset) as *mut T }
}

#[inline(always)]
pub unsafe fn read_unaligned<T: Copy>(ptr: *const T) -> T {
    core::ptr::read_unaligned(ptr)
}

#[inline(always)]
pub fn is_valid_ptr(addr: usize) -> bool {
    addr > 0x10000 && addr < 0x7FFFFFFFFFFF
}
