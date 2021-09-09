#![allow(dead_code)]

// https://emberian.github.io/evdev/src/ioctl/platform/linux.rs.html#410-412
// but with cosnt fn instead of macros

// and some part from linux/source/include/uapi/linux/input.h

#[doc(hidden)]
pub const NRBITS: u32 = 8;
#[doc(hidden)]
pub const TYPEBITS: u32 = 8;

#[cfg(target_arch = "mips")]
mod consts {
    #[doc(hidden)]
    pub const NONE: u8 = 1;
    #[doc(hidden)]
    pub const READ: u8 = 2;
    #[doc(hidden)]
    pub const WRITE: u8 = 4;
    #[doc(hidden)]
    pub const SIZEBITS: u8 = 13;
    #[doc(hidden)]
    pub const DIRBITS: u8 = 3;
}
#[cfg(target_arch = "powerpc")]
mod consts {
    #[doc(hidden)]
    pub const NONE: u8 = 1;
    #[doc(hidden)]
    pub const READ: u8 = 2;
    #[doc(hidden)]
    pub const WRITE: u8 = 4;
    #[doc(hidden)]
    pub const SIZEBITS: u8 = 13;
    #[doc(hidden)]
    pub const DIRBITS: u8 = 3;
}

#[cfg(not(any(
    target_arch = "powerpc",
    target_arch = "mips",
    target_arch = "x86",
    target_arch = "arm",
    target_arch = "x86_64",
    target_arch = "aarch64"
)))]
use this_arch_not_supported;

// "Generic" ioctl protocol
#[cfg(any(
    target_arch = "x86",
    target_arch = "arm",
    target_arch = "x86_64",
    target_arch = "aarch64"
))]
mod consts {
    #[doc(hidden)]
    pub const NONE: u8 = 0;
    #[doc(hidden)]
    pub const READ: u8 = 2;
    #[doc(hidden)]
    pub const WRITE: u8 = 1;
    #[doc(hidden)]
    pub const SIZEBITS: u8 = 14;
    #[doc(hidden)]
    pub const DIRBITS: u8 = 2;
}

#[doc(hidden)]
pub use self::consts::*;

#[doc(hidden)]
pub const NRSHIFT: u32 = 0;
#[doc(hidden)]
pub const TYPESHIFT: u32 = NRSHIFT + NRBITS as u32;
#[doc(hidden)]
pub const SIZESHIFT: u32 = TYPESHIFT + TYPEBITS as u32;
#[doc(hidden)]
pub const DIRSHIFT: u32 = SIZESHIFT + SIZEBITS as u32;

#[doc(hidden)]
pub const NRMASK: u32 = (1 << NRBITS) - 1;
#[doc(hidden)]
pub const TYPEMASK: u32 = (1 << TYPEBITS) - 1;
#[doc(hidden)]
pub const SIZEMASK: u32 = (1 << SIZEBITS) - 1;
#[doc(hidden)]
pub const DIRMASK: u32 = (1 << DIRBITS) - 1;

/// Encode an ioctl command.
pub const fn ioc(dir: u64, ty: u64, nr: u64, sz: u64) -> u64 {
    (((dir as u32) << DIRSHIFT)
        | ((ty as u32) << TYPESHIFT)
        | ((nr as u32) << NRSHIFT)
        | ((sz as u32) << SIZESHIFT)) as u64
}

/// Encode an ioctl command that has no associated data.
pub const fn io(ty: u64, nr: u64) -> u64 {
    ioc(NONE as _, ty, nr, 0)
}

/// Encode an ioctl command that reads.
pub const fn ior(ty: u64, nr: u64, sz: u64) -> u64 {
    ioc(READ as _, ty, nr, sz)
}

/// Encode an ioctl command that writes.
pub const fn iow(ty: u64, nr: u64, sz: u64) -> u64 {
    ioc(WRITE as _, ty, nr, sz)
}

/// Encode an ioctl command that both reads and writes.
pub const fn iorw(ty: u64, nr: u64, sz: u64) -> u64 {
    ioc((READ | WRITE) as _, ty, nr, sz)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct InputAbsInfo {
    pub value: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub fuzz: i32,
    pub flat: i32,
    pub resolution: i32,
}
impl Default for InputAbsInfo {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct InputId {
    pub bustype: u16,
    pub vendor: u16,
    pub product: u16,
    pub version: u16,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct TimeVal {
    pub tv_sec: libc::time_t,
    pub tv_usec: libc::suseconds_t,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct InputEvent {
    pub time: TimeVal,
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

pub const fn eviocgid() -> u64 {
    ior(b'E' as _, 0x02, std::mem::size_of::<InputId>() as _)
}

pub const fn eviocgbit(ev: u32, len: u32) -> u64 {
    ioc(READ as _, b'E' as _, 0x20 + (ev as u64), len as _)
}

pub const fn eviocgname(len: u32) -> u64 {
    ioc(READ as _, b'E' as _, 0x06, len as _)
}

pub const fn eviocgabs(abs: u32) -> u64 {
    ior(
        b'E' as _,
        (0x40 + abs) as _,
        std::mem::size_of::<InputAbsInfo>() as _,
    )
}
