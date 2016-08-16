/* TOOD: Implement for other kqueue based systems
 */

use {Errno, Result};
#[cfg(not(target_os = "netbsd"))]
use libc::{timespec, time_t, c_int, c_long, intptr_t, uintptr_t};
#[cfg(target_os = "netbsd")]
use libc::{timespec, time_t, c_long, intptr_t, uintptr_t, size_t};
use libc;
use std::os::unix::io::RawFd;
use std::ptr;

// Redefine kevent in terms of programmer-friendly enums and bitfields.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct KEvent {
    pub ident: uintptr_t,
    pub filter: EventFilter,
    pub flags: EventFlag,
    pub fflags: FilterFlag,
    pub data: intptr_t,
    // libc defines udata as a pointer on most OSes.  But it's really
    // more like an arbitrary tag
    pub udata: uintptr_t
}

#[cfg(not(target_os = "netbsd"))]
#[repr(i16)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventFilter {
    EVFILT_AIO = libc::EVFILT_AIO,
    #[cfg(target_os = "dragonfly")]
    EVFILT_EXCEPT = libc::EVFILT_EXCEPT,
    #[cfg(any(target_os = "macos",
              target_os = "dragonfly",
              target_os = "freebsd"))]
    EVFILT_FS = libc::EVFILT_FS,
    #[cfg(target_os = "freebsd")]
    EVFILT_LIO = libc::EVFILT_LIO,
    #[cfg(target_os = "macos")]
    EVFILT_MACHPORT = libc::EVFILT_MACHPORT,
    EVFILT_PROC = libc::EVFILT_PROC,
    #[cfg(target_os = "freebsd")]
    EVFILT_PROCDESC = libc::EVFILT_PROCDESC,
    EVFILT_READ = libc::EVFILT_READ,
    #[cfg(target_os = "freebsd")]
    EVFILT_SENDFILE = libc::EVFILT_SENDFILE,
    EVFILT_SIGNAL = libc::EVFILT_SIGNAL,
    EVFILT_SYSCOUNT = libc::EVFILT_SYSCOUNT,
    EVFILT_TIMER = libc::EVFILT_TIMER,
    #[cfg(any(target_os = "macos",
              target_os = "dragonfly",
              target_os = "freebsd"))]
    EVFILT_USER = libc::EVFILT_USER,
    #[cfg(target_os = "macos")]
    EVFILT_VM = libc::EVFILT_VM,
    EVFILT_VNODE = libc::EVFILT_VNODE,
    EVFILT_WRITE = libc::EVFILT_WRITE,
}

#[cfg(target_os = "netbsd")]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventFilter {
    EVFILT_READ = libc::EVFILT_READ,
    EVFILT_WRITE = libc::EVFILT_WRITE,
    EVFILT_AIO = libc::EVFILT_AIO,
    EVFILT_VNODE = libc::EVFILT_VNODE,
    EVFILT_PROC = libc::EVFILT_PROC,
    EVFILT_SIGNAL = libc::EVFILT_SIGNAL,
    EVFILT_TIMER = libc::EVFILT_TIMER,
    EVFILT_SYSCOUNT = libc::EVFILT_SYSCOUNT,
}

#[cfg(any(target_os = "macos",
          target_os = "freebsd",
          target_os = "dragonfly"))]
bitflags!(
    flags EventFlag: u16 {
        const EV_ADD       = libc::EV_ADD,
        const EV_CLEAR     = libc::EV_CLEAR,
        const EV_DELETE    = libc::EV_DELETE,
        const EV_DISABLE   = libc::EV_DISABLE,
        const EV_DISPATCH  = libc::EV_DISPATCH,
        #[cfg(target_os = "freebsd")]
        const EV_DROP      = libc::EV_DROP,
        const EV_ENABLE    = libc::EV_ENABLE,
        const EV_EOF       = libc::EV_EOF,
        const EV_ERROR     = libc::EV_ERROR,
        #[cfg(target_os = "macos")]
        const EV_FLAG0     = libc::EV_FLAG0,
        const EV_FLAG1     = libc::EV_FLAG1,
        #[cfg(target_os = "freebsd")]
        const EV_FLAG2     = libc::EV_FLAG2,
        #[cfg(target_os = "freebsd")]
        const EV_FORCEONESHOT = libc::EV_FORCEONESHOT,
        #[cfg(target_os = "dragonfly")]
        const EV_NODATA    = libc::EV_NODATA,
        const EV_ONESHOT   = libc::EV_ONESHOT,
        #[cfg(target_os = "macos")]
        const EV_OOBAND    = libc::EV_OOBAND,
        #[cfg(target_os = "macos")]
        const EV_POLL      = libc::EV_POLL,
        const EV_RECEIPT   = libc::EV_RECEIPT,
        const EV_SYSFLAGS  = libc::EV_SYSFLAGS,
    }
);

#[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
bitflags!(
    flags EventFlag: u32 {
        const EV_ADD       = libc::EV_ADD,
        const EV_DELETE    = libc::EV_DELETE,
        const EV_ENABLE    = libc::EV_ENABLE,
        const EV_DISABLE   = libc::EV_DISABLE,
        const EV_ONESHOT   = libc::EV_ONESHOT,
        const EV_CLEAR     = libc::EV_CLEAR,
        #[cfg(target_os = "openbsd")]
        const EV_FLAG1     = libc::EV_FLAG1,
        #[cfg(target_os = "netbsd")]
        const EV_RECEIPT   = libc::EV_RECEIPT,
        const EV_DISPATCH  = libc::EV_DISPATCH,
        const EV_SYSFLAGS  = libc::EV_SYSFLAGS,
        const EV_FLAG1     = libc::EV_FLAG1,
        const EV_EOF       = libc::EV_EOF,
        const EV_ERROR     = libc::EV_ERROR,
    }
);

bitflags!(
    flags FilterFlag: u32 {
        #[cfg(target_os = "macos")]
        const NOTE_ABSOLUTE                        = libc::NOTE_ABSOLUTE,
        #[cfg(target_os = "macos")]
        const NOTE_APPACTIVE                       = libc::NOTE_APPACTIVE,
        #[cfg(target_os = "macos")]
        const NOTE_APPALLSTATES                    = libc::NOTE_APPALLSTATES,
        #[cfg(target_os = "macos")]
        const NOTE_APPBACKGROUND                   = libc::NOTE_APPBACKGROUND,
        #[cfg(target_os = "macos")]
        const NOTE_APPINACTIVE                     = libc::NOTE_APPINACTIVE,
        #[cfg(target_os = "macos")]
        const NOTE_APPNONUI                        = libc::NOTE_APPNONUI,
        const NOTE_ATTRIB                          = libc::NOTE_ATTRIB,
        const NOTE_CHILD                           = libc::NOTE_CHILD,
        #[cfg(target_os = "freebsd")]
        const NOTE_CLOSE                           = libc::NOTE_CLOSE,
        #[cfg(target_os = "freebsd")]
        const NOTE_CLOSE_WRITE                     = libc::NOTE_CLOSE_WRITE,
        const NOTE_DELETE                          = libc::NOTE_DELETE,
        #[cfg(target_os = "openbsd")]
        const NOTE_EOF                             = libc::NOTE_EOF,
        const NOTE_EXEC                            = libc::NOTE_EXEC,
        const NOTE_EXIT                            = libc::NOTE_EXIT,
        #[cfg(target_os = "macos")]
        const NOTE_EXIT_REPARENTED                 = libc::NOTE_EXIT_REPARENTED,
        #[cfg(target_os = "macos")]
        const NOTE_EXITSTATUS                      = libc::NOTE_EXITSTATUS,
        const NOTE_EXTEND                          = libc::NOTE_EXTEND,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFAND                           = libc::NOTE_FFAND,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFCOPY                          = libc::NOTE_FFCOPY,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFCTRLMASK                      = libc::NOTE_FFCTRLMASK,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFLAGSMASK                      = libc::NOTE_FFLAGSMASK,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFNOP                           = libc::NOTE_FFNOP,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFOR                            = libc::NOTE_FFOR,
        #[cfg(target_os = "freebsd")]
        const NOTE_FILE_POLL                       = libc::NOTE_FILE_POLL,
        const NOTE_FORK                            = libc::NOTE_FORK,
        const NOTE_LINK                            = libc::NOTE_LINK,
        const NOTE_LOWAT                           = libc::NOTE_LOWAT,
        #[cfg(target_os = "freebsd")]
        const NOTE_MSECONDS                        = libc::NOTE_MSECONDS,
        #[cfg(target_os = "macos")]
        const NOTE_NONE                            = libc::NOTE_NONE,
        #[cfg(any(target_os = "macos", target_os = "freebsd"))]
        const NOTE_NSECONDS                        = libc::NOTE_NSECONDS,
        #[cfg(target_os = "dragonfly")]
        const NOTE_OOB                             = libc::NOTE_OOB,
        #[cfg(target_os = "freebsd")]
        const NOTE_OPEN                            = libc::NOTE_OPEN,
        const NOTE_PCTRLMASK                       = libc::NOTE_PCTRLMASK,
        const NOTE_PDATAMASK                       = libc::NOTE_PDATAMASK,
        #[cfg(target_os = "freebsd")]
        const NOTE_READ                            = libc::NOTE_READ,
        #[cfg(target_os = "macos")]
        const NOTE_REAP                            = libc::NOTE_REAP,
        const NOTE_RENAME                          = libc::NOTE_RENAME,
        #[cfg(target_os = "macos")]
        const NOTE_RESOURCEEND                     = libc::NOTE_RESOURCEEND,
        const NOTE_REVOKE                          = libc::NOTE_REVOKE,
        #[cfg(any(target_os = "macos", target_os = "freebsd"))]
        const NOTE_SECONDS                         = libc::NOTE_SECONDS,
        #[cfg(target_os = "macos")]
        const NOTE_SIGNAL                          = libc::NOTE_SIGNAL,
        const NOTE_TRACK                           = libc::NOTE_TRACK,
        const NOTE_TRACKERR                        = libc::NOTE_TRACKERR,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_TRIGGER                         = libc::NOTE_TRIGGER,
        #[cfg(target_os = "openbsd")]
        const NOTE_TRUNCATE                        = libc::NOTE_TRUNCATE,
        #[cfg(any(target_os = "macos", target_os = "freebsd"))]
        const NOTE_USECONDS                        = libc::NOTE_USECONDS,
        #[cfg(target_os = "macos")]
        const NOTE_VM_ERROR                        = libc::NOTE_VM_ERROR,
        #[cfg(target_os = "macos")]
        const NOTE_VM_PRESSURE                     = libc::NOTE_VM_PRESSURE,
        #[cfg(target_os = "macos")]
        const NOTE_VM_PRESSURE_SUDDEN_TERMINATE    = libc::NOTE_VM_PRESSURE_SUDDEN_TERMINATE,
        #[cfg(target_os = "macos")]
        const NOTE_VM_PRESSURE_TERMINATE           = libc::NOTE_VM_PRESSURE_TERMINATE,
        const NOTE_WRITE                           = libc::NOTE_WRITE,
    }
);

pub fn kqueue() -> Result<RawFd> {
    let res = unsafe { libc::kqueue() };

    Errno::result(res)
}

pub fn kevent(kq: RawFd,
              changelist: &[KEvent],
              eventlist: &mut [KEvent],
              timeout_ms: usize) -> Result<usize> {

    // Convert ms to timespec
    let timeout = timespec {
        tv_sec: (timeout_ms / 1000) as time_t,
        tv_nsec: ((timeout_ms % 1000) * 1_000_000) as c_long
    };

    kevent_ts(kq, changelist, eventlist, Some(timeout))
}

#[cfg(any(target_os = "macos",
          target_os = "freebsd",
          target_os = "dragonfly"))]
pub fn kevent_ts(kq: RawFd,
              changelist: &[KEvent],
              eventlist: &mut [KEvent],
              timeout_opt: Option<timespec>) -> Result<usize> {

    let res = unsafe {
        libc::kevent(
            kq,
            changelist.as_ptr() as *const libc::kevent,
            changelist.len() as c_int,
            eventlist.as_mut_ptr() as *mut libc::kevent,
            eventlist.len() as c_int,
            if let Some(ref timeout) = timeout_opt {timeout as *const timespec} else {ptr::null()})
    };

    Errno::result(res).map(|r| r as usize)
}

#[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
pub fn kevent_ts(kq: RawFd,
              changelist: &[KEvent],
              eventlist: &mut [KEvent],
              timeout_opt: Option<timespec>) -> Result<usize> {

    let res = unsafe {
        libc::kevent(
            kq,
            changelist.as_ptr() as *const libc::kevent,
            changelist.len() as size_t,
            eventlist.as_mut_ptr() as *mut libc::kevent,
            eventlist.len() as size_t,
            if let Some(ref timeout) = timeout_opt {timeout as *const timespec} else {ptr::null()})
    };

    Errno::result(res).map(|r| r as usize)
}

#[inline]
pub fn ev_set(ev: &mut KEvent,
              ident: usize,
              filter: EventFilter,
              flags: EventFlag,
              fflags: FilterFlag,
              udata: uintptr_t) {

    ev.ident  = ident as uintptr_t;
    ev.filter = filter;
    ev.flags  = flags;
    ev.fflags = fflags;
    ev.data   = 0;
    ev.udata  = udata;
}
