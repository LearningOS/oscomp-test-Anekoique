use arceos_posix_api as api;
use axerrno::{LinuxError, LinuxResult};
use axprocess::Pid;
use axtask::{TaskExtRef, current};
use core::ffi::c_int;
use linux_raw_sys::general::{RLIM_NLIMITS, rlimit64};
use starry_core::task::{ProcessData, get_process};

use crate::ptr::{PtrWrapper, UserConstPtr, UserPtr};

/// Gets resource limits for the calling process.
///
/// This syscall retrieves the soft and hard limits for a system resource.
///
/// # Arguments
///
/// * `resource` - The resource identifier (RLIMIT_CPU, RLIMIT_NOFILE, etc.)
/// * `rlimit` - Pointer to a structure where the current limits will be stored
///
/// # Returns
///
/// * `Ok(0)` on success
/// * Error code on failure (e.g., EINVAL for invalid resource, EFAULT for invalid pointer)
///
/// # Safety
///
/// The rlimit pointer must point to valid, writable memory or be NULL.
pub fn sys_getrlimit(resource: c_int, rlimit: UserPtr<api::ctypes::rlimit>) -> LinuxResult<isize> {
    if let Some(rlimit) = rlimit.nullable(|rlimit| rlimit.get())? {
        Ok(unsafe { api::sys_getrlimit(resource, rlimit) as _ })
    } else {
        Err(LinuxError::EFAULT)
    }
}

/// Sets resource limits for the calling process.
///
/// This syscall updates the soft and hard limits for a system resource.
/// Unprivileged processes may only lower their hard limits, not raise them.
///
/// # Arguments
///
/// * `resource` - The resource identifier (RLIMIT_CPU, RLIMIT_NOFILE, etc.)
/// * `rlimit` - Pointer to a structure containing the new limits to set
///
/// # Returns
///
/// * `Ok(0)` on success
/// * Error code on failure (e.g., EINVAL for invalid resource, EPERMuuur permission issues)
///
/// # Safety
///
/// The rlimit pointer must point to valid, readable memory or be NULL.
pub fn sys_setrlimit(resource: c_int, rlimit: UserPtr<api::ctypes::rlimit>) -> LinuxResult<isize> {
    if let Some(rlimit) = rlimit.nullable(|rlimit| rlimit.get())? {
        Ok(unsafe { api::sys_setrlimit(resource, rlimit) as _ })
    } else {
        Err(LinuxError::EFAULT)
    }
}

/// Gets or sets resource limits for a process, specified by PID.
///
/// This syscall combines functionality of getrlimit and setrlimit, with additional
/// ability to target a specific process by its PID.
///
/// # Arguments
///
/// * `pid` - Process ID (0 for current process)
/// * `resource` - The resource identifier (RLIMIT_CPU, RLIMIT_NOFILE, etc.)
/// * `new_limit` - Pointer to new limits to set (or NULL for no change)
/// * `old_limit` - Pointer where current limits will be stored (or NULL if not needed)
///
/// # Returns
///
/// * `Ok(0)` on success
/// * Error code on failure (e.g., ESRCH for invalid PID, EPERM for permission issues)
pub fn sys_prlimit64(
    pid: Pid,
    resource: u32,
    new_limit: UserConstPtr<rlimit64>,
    old_limit: UserPtr<rlimit64>,
) -> LinuxResult<isize> {
    if resource >= RLIM_NLIMITS {
        return Err(LinuxError::EINVAL);
    }

    let proc = if pid == 0 {
        current().task_ext().thread.process().clone()
    } else {
        get_process(pid)?
    };

    let proc_data: &ProcessData = proc.data().unwrap();
    if let Some(old_limit) = old_limit.nullable(|old_limit| old_limit.get())? {
        let limit = &proc_data.rlimits.read()[resource];
        unsafe {
            (*old_limit).rlim_cur = limit.current;
            (*old_limit).rlim_max = limit.max;
        }
    }

    if let Some(new_limit) = new_limit.nullable(|new_limit| new_limit.get())? {
        if unsafe { (*new_limit).rlim_cur } > unsafe { (*new_limit).rlim_max } {
            return Err(LinuxError::EINVAL);
        }

        let limit = &mut proc_data.rlimits.write()[resource];
        if unsafe { (*new_limit).rlim_max } <= limit.max {
            limit.max = unsafe { (*new_limit).rlim_max };
        } else {
            return Err(LinuxError::EPERM);
        }

        limit.current = unsafe { (*new_limit).rlim_cur };
    }

    Ok(0)
}
