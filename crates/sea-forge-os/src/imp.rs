//! Windows implementation. Each `unsafe` block names the invariant it relies on.

use std::ffi::c_void;
use std::io;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::process::Child;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{LocalFree, HANDLE};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, RevertToSelf, SecurityImpersonation, TokenImpersonationLevel, TokenUser,
    PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicAccountingInformation,
    JobObjectExtendedLimitInformation, QueryInformationJobObject, SetInformationJobObject,
    TerminateJobObject, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::Pipes::{GetNamedPipeServerProcessId, ImpersonateNamedPipeClient};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, OpenProcess, OpenProcessToken, OpenThreadToken,
    PROCESS_QUERY_LIMITED_INFORMATION,
};

const TRUE: i32 = 1;
const FALSE: i32 = 0;

fn last_error() -> io::Error {
    io::Error::last_os_error()
}

/// Accepts only the `S-1-<digits>-<digits>...` grammar, so a SID can be
/// spliced into SDDL without any possibility of injecting ACE or flag syntax.
fn check_sid_grammar(sid: &str) -> io::Result<()> {
    let malformed = || io::Error::new(io::ErrorKind::InvalidInput, "malformed SID string");
    let body = sid.strip_prefix("S-1-").ok_or_else(malformed)?;
    if body.is_empty() || sid.len() > 256 {
        return Err(malformed());
    }
    for part in body.split('-') {
        if part.is_empty() || part.len() > 20 || !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(malformed());
        }
    }
    Ok(())
}

/// Converts a NUL-terminated UTF-16 string allocated by Windows.
fn wide_ptr_to_string(ptr: *const u16) -> io::Result<String> {
    let mut len = 0usize;
    // SAFETY: the caller passes a non-null pointer to a NUL-terminated wide
    // string; every index read is before the terminator found here.
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
    }
    // SAFETY: `len` code units were just read and lie inside the same allocation.
    let units = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf16(units).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "SID is not UTF-16"))
}

fn sid_to_string(sid: *mut c_void) -> io::Result<String> {
    let mut out: *mut u16 = null_mut();
    // SAFETY: `sid` points at a valid SID inside a live buffer owned by the caller;
    // `out` is a valid out-pointer that receives a LocalAlloc'd string on success.
    if unsafe { ConvertSidToStringSidW(sid, &mut out) } == FALSE {
        return Err(last_error());
    }
    let text = wide_ptr_to_string(out);
    // SAFETY: `out` was allocated by ConvertSidToStringSidW and is freed exactly once.
    unsafe { LocalFree(out.cast()) };
    text
}

/// SID string of the user recorded in a token.
fn token_user_sid(token: HANDLE) -> io::Result<String> {
    let mut needed: u32 = 0;
    // SAFETY: a null buffer with length 0 is the documented size query; `needed` is a valid out-pointer.
    unsafe { GetTokenInformation(token, TokenUser, null_mut(), 0, &mut needed) };
    if needed == 0 {
        return Err(last_error());
    }
    // u64 storage keeps the buffer 8-byte aligned for TOKEN_USER.
    let mut buffer = vec![0u64; (needed as usize).div_ceil(8)];
    // SAFETY: `buffer` holds at least `needed` bytes and is 8-byte aligned; the kernel writes a TOKEN_USER into it.
    let ok = unsafe {
        GetTokenInformation(token, TokenUser, buffer.as_mut_ptr().cast(), needed, &mut needed)
    };
    if ok == FALSE {
        return Err(last_error());
    }
    // SAFETY: on success the buffer starts with a TOKEN_USER; `User.Sid` points into the same buffer, which outlives this call.
    let user = unsafe { &*(buffer.as_ptr() as *const TOKEN_USER) };
    sid_to_string(user.User.Sid)
}

/// Reverts the calling thread to its own token when dropped.
struct Impersonation;

impl Drop for Impersonation {
    fn drop(&mut self) {
        // SAFETY: RevertToSelf takes no arguments and only affects the calling thread.
        if unsafe { RevertToSelf() } == FALSE {
            // A thread left impersonating a remote client is a privilege leak;
            // there is no safe way to continue.
            std::process::abort();
        }
    }
}

/// SID of the connecting client of a named pipe server handle.
///
/// The client must already have written at least one byte: impersonation
/// fails with `ERROR_CANNOT_IMPERSONATE` before the first read. The result is
/// read from the kernel token, so it cannot be spoofed by a reused process id.
pub fn pipe_client_sid(pipe: &impl AsRawHandle) -> io::Result<String> {
    let handle = pipe.as_raw_handle() as HANDLE;
    // SAFETY: the kernel validates the handle; an invalid one makes the call fail.
    if unsafe { ImpersonateNamedPipeClient(handle) } == FALSE {
        return Err(last_error());
    }
    // Declared before the token so it drops last: the thread reverts after the token is closed.
    let _revert = Impersonation;
    let mut raw: HANDLE = null_mut();
    // SAFETY: GetCurrentThread returns a pseudo-handle; OpenAsSelf=TRUE queries the thread token with the process's own rights; `raw` is a valid out-pointer.
    if unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, TRUE, &mut raw) } == FALSE {
        return Err(last_error());
    }
    // SAFETY: OpenThreadToken succeeded, so `raw` is a fresh handle owned by nobody else.
    let token = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    let mut level: i32 = 0;
    let mut needed: u32 = 0;
    // SAFETY: `level` is a 4-byte out buffer matching the size passed; `needed` is a valid out-pointer.
    let ok = unsafe {
        GetTokenInformation(
            token.as_raw_handle() as HANDLE,
            TokenImpersonationLevel,
            (&mut level as *mut i32).cast(),
            mem::size_of::<i32>() as u32,
            &mut needed,
        )
    };
    if ok == FALSE {
        return Err(last_error());
    }
    if level < SecurityImpersonation {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "pipe client did not grant impersonation",
        ));
    }
    token_user_sid(token.as_raw_handle() as HANDLE)
}

/// SID of the process that owns the server end of a named pipe.
/// The client compares it with the expected owner before sending anything.
pub fn pipe_server_sid(pipe: &impl AsRawHandle) -> io::Result<String> {
    let handle = pipe.as_raw_handle() as HANDLE;
    let mut pid: u32 = 0;
    // SAFETY: the kernel validates the handle; `pid` is a valid out-pointer.
    if unsafe { GetNamedPipeServerProcessId(handle, &mut pid) } == FALSE {
        return Err(last_error());
    }
    // SAFETY: plain call with no pointers; a null return is checked below.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid) };
    if process.is_null() {
        return Err(last_error());
    }
    // SAFETY: OpenProcess returned a fresh, non-null handle we own.
    let process = unsafe { OwnedHandle::from_raw_handle(process as _) };
    let mut raw: HANDLE = null_mut();
    // SAFETY: the process handle is live; `raw` is a valid out-pointer.
    if unsafe { OpenProcessToken(process.as_raw_handle() as HANDLE, TOKEN_QUERY, &mut raw) } == FALSE {
        return Err(last_error());
    }
    // SAFETY: OpenProcessToken succeeded, so `raw` is a fresh handle we own.
    let token = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    token_user_sid(token.as_raw_handle() as HANDLE)
}

/// SID of the user running this process.
pub fn current_sid() -> io::Result<String> {
    let mut raw: HANDLE = null_mut();
    // SAFETY: GetCurrentProcess returns a pseudo-handle; `raw` is a valid out-pointer.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) } == FALSE {
        return Err(last_error());
    }
    // SAFETY: OpenProcessToken succeeded, so `raw` is a fresh handle we own.
    let token = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    token_user_sid(token.as_raw_handle() as HANDLE)
}

/// Pipe DACL: SYSTEM and the current user get full control; each bound SID
/// gets read and write only. Everyone else is denied by omission.
pub struct PipeSecurity {
    descriptor: PSECURITY_DESCRIPTOR,
    attributes: SECURITY_ATTRIBUTES,
}

impl PipeSecurity {
    /// Builds the DACL. Every SID is grammar-checked before it reaches SDDL.
    pub fn new(sids: &[String]) -> io::Result<Self> {
        let current = current_sid()?;
        let mut sddl = format!("D:P(A;;GA;;;SY)(A;;GA;;;{current})");
        for sid in sids {
            check_sid_grammar(sid)?;
            sddl.push_str(&format!("(A;;GRGW;;;{sid})"));
        }
        let wide: Vec<u16> = std::ffi::OsStr::new(&sddl)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut descriptor: PSECURITY_DESCRIPTOR = null_mut();
        // SAFETY: `wide` is NUL-terminated and outlives the call; `descriptor` is a valid out-pointer; the size out-pointer is optional and null here.
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                null_mut(),
            )
        };
        if ok == FALSE {
            return Err(last_error());
        }
        Ok(Self {
            descriptor,
            attributes: SECURITY_ATTRIBUTES {
                nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: descriptor,
                bInheritHandle: FALSE,
            },
        })
    }

    /// Pointer to the `SECURITY_ATTRIBUTES`, valid while `self` is alive.
    pub fn attributes_ptr(&self) -> *mut c_void {
        (&self.attributes as *const SECURITY_ATTRIBUTES)
            .cast_mut()
            .cast()
    }
}

impl Drop for PipeSecurity {
    fn drop(&mut self) {
        // SAFETY: the descriptor came from ConvertStringSecurityDescriptorToSecurityDescriptorW (LocalAlloc) and is freed exactly once.
        unsafe { LocalFree(self.descriptor.cast()) };
    }
}

/// Creates one named pipe server instance. `first` maps to
/// `FILE_FLAG_FIRST_PIPE_INSTANCE`: an existing pipe of that name makes the
/// call fail, so a squatter cannot be silently joined.
#[cfg(feature = "tokio-pipes")]
pub fn create_pipe_server(
    name: &str,
    security: &PipeSecurity,
    first: bool,
) -> io::Result<tokio::net::windows::named_pipe::NamedPipeServer> {
    let mut options = tokio::net::windows::named_pipe::ServerOptions::new();
    options.first_pipe_instance(first).reject_remote_clients(true);
    // SAFETY: `attributes_ptr` points at a SECURITY_ATTRIBUTES whose descriptor stays alive for the borrow of `security`; tokio passes it to CreateNamedPipeW synchronously and keeps no reference.
    unsafe { options.create_with_security_attributes_raw(name, security.attributes_ptr()) }
}

/// A Job Object. Processes assigned to it are terminated together.
pub struct Job {
    handle: OwnedHandle,
}

impl Job {
    /// Creates a job whose processes are killed when the last handle closes.
    pub fn kill_on_close() -> io::Result<Job> {
        // SAFETY: null security attributes and name are valid arguments.
        let raw = unsafe { CreateJobObjectW(null(), null()) };
        if raw.is_null() {
            return Err(last_error());
        }
        // SAFETY: CreateJobObjectW returned a fresh, non-null handle we own.
        let handle = unsafe { OwnedHandle::from_raw_handle(raw as _) };
        // SAFETY: JOBOBJECT_EXTENDED_LIMIT_INFORMATION is plain data; all-zero is valid.
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { mem::zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `info` is a live struct of the size passed for the given information class.
        let ok = unsafe {
            SetInformationJobObject(
                handle.as_raw_handle() as HANDLE,
                JobObjectExtendedLimitInformation,
                (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if ok == FALSE {
            return Err(last_error());
        }
        Ok(Job { handle })
    }

    /// Places a spawned child in the job. Grandchildren created before this
    /// call are not in the job; spawn, then assign, before the child runs far.
    pub fn assign(&self, child: &Child) -> io::Result<()> {
        // SAFETY: both handles are live: the job is owned by `self`, the process by `child`.
        let ok = unsafe {
            AssignProcessToJobObject(
                self.handle.as_raw_handle() as HANDLE,
                child.as_raw_handle() as HANDLE,
            )
        };
        if ok == FALSE {
            return Err(last_error());
        }
        Ok(())
    }

    /// Terminates every process in the job.
    pub fn terminate(&self) -> io::Result<()> {
        // SAFETY: the job handle is live; exit code 1 is an arbitrary non-zero value.
        if unsafe { TerminateJobObject(self.handle.as_raw_handle() as HANDLE, 1) } == FALSE {
            return Err(last_error());
        }
        Ok(())
    }

    /// Number of processes still active in the job.
    pub fn active_processes(&self) -> io::Result<u32> {
        // SAFETY: plain data struct; all-zero is valid and overwritten on success.
        let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { mem::zeroed() };
        // SAFETY: `info` is live and matches the size passed for the information class; no return-length pointer is needed.
        let ok = unsafe {
            QueryInformationJobObject(
                self.handle.as_raw_handle() as HANDLE,
                JobObjectBasicAccountingInformation,
                (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                mem::size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                null_mut(),
            )
        };
        if ok == FALSE {
            return Err(last_error());
        }
        Ok(info.ActiveProcesses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    #[test]
    fn current_sid_is_a_user_sid() {
        let sid = current_sid().expect("current sid");
        assert!(sid.starts_with("S-1-5-"), "{sid}");
        check_sid_grammar(&sid).expect("grammar");
    }

    #[test]
    fn pipe_security_rejects_malformed_sids() {
        for bad in ["", "S-1-", "S-1-5-21-x", "S-1-5-21-1)(A;;GA;;;WD", "BU", "S-1-5--21", "S-1-5-21 "] {
            let err = PipeSecurity::new(&[bad.to_string()]).err();
            assert!(err.is_some(), "accepted {bad:?}");
        }
    }

    #[test]
    fn pipe_security_accepts_well_formed_sid() {
        let sid = current_sid().expect("current sid");
        PipeSecurity::new(&[sid]).expect("well-formed sid builds a descriptor");
    }

    #[test]
    fn job_terminates_child_tree() {
        let job = Job::kill_on_close().expect("job");
        let mut child = Command::new("cmd")
            .args(["/c", "ping", "-n", "30", "127.0.0.1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn");
        job.assign(&child).expect("assign");
        assert!(job.active_processes().expect("count") >= 1);
        job.terminate().expect("terminate");
        let _ = child.wait();
        let deadline = Instant::now() + Duration::from_secs(10);
        while job.active_processes().expect("count") != 0 {
            assert!(Instant::now() < deadline, "processes survived termination");
            std::thread::sleep(Duration::from_millis(50));
        }
    }

}
