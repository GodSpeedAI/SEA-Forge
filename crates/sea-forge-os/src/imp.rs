//! Windows implementation. Each `unsafe` block names the invariant it relies on.

use std::ffi::{c_void, OsStr};
use std::io;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::ptr::{null, null_mut};
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{
    LocalFree, ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND,
    GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, RevertToSelf, SecurityIdentification, TokenImpersonationLevel, TokenUser,
    PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateDirectoryW, CreateFileW, FileDispositionInfo, FileRenameInfo, GetFileInformationByHandle,
    GetFinalPathNameByHandleW, SetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, CREATE_NEW,
    DELETE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_DISPOSITION_INFO, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_READ_ATTRIBUTES, FILE_RENAME_INFO, FILE_RENAME_INFO_0, FILE_SHARE_DELETE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING, VOLUME_NAME_DOS,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicAccountingInformation,
    JobObjectExtendedLimitInformation, QueryInformationJobObject, SetInformationJobObject,
    TerminateJobObject, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::Pipes::{GetNamedPipeServerProcessId, ImpersonateNamedPipeClient};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, OpenProcess, OpenProcessToken, OpenThread,
    OpenThreadToken, ResumeThread, CREATE_SUSPENDED, PROCESS_QUERY_LIMITED_INFORMATION,
    THREAD_SUSPEND_RESUME,
};

const TRUE: i32 = 1;
const FALSE: i32 = 0;

fn last_error() -> io::Error {
    io::Error::last_os_error()
}

/// NUL-terminated UTF-16 copy of `s`, as the wide-string APIs expect.
fn to_wide(s: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    s.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

/// Primary token of `process`, opened for query.
fn process_token(process: HANDLE) -> io::Result<OwnedHandle> {
    let mut raw: HANDLE = null_mut();
    // SAFETY: `process` is a live process handle or pseudo-handle; `raw` is a valid out-pointer.
    if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut raw) } == FALSE {
        return Err(last_error());
    }
    // SAFETY: OpenProcessToken succeeded, so `raw` is a fresh handle we own.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw as _) })
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
    String::from_utf16(units)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "SID is not UTF-16"))
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
        GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
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
    // Identification is enough to read TokenUser, which is all the server
    // needs. Admitting identification-only clients adds no authority: they
    // cannot be impersonated into the client's own access rights.
    if level < SecurityIdentification {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "pipe client did not grant identification",
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
    let token = process_token(process.as_raw_handle() as HANDLE)?;
    token_user_sid(token.as_raw_handle() as HANDLE)
}

/// SID of the user running this process.
pub fn current_sid() -> io::Result<String> {
    // SAFETY: GetCurrentProcess returns a pseudo-handle that needs no close.
    let token = process_token(unsafe { GetCurrentProcess() })?;
    token_user_sid(token.as_raw_handle() as HANDLE)
}

/// Rights granted to each bound client SID on the pipe: `FILE_GENERIC_READ`
/// (includes SYNCHRONIZE) plus `FILE_WRITE_DATA` and `FILE_WRITE_ATTRIBUTES`.
/// `GENERIC_WRITE` is deliberately not used: its file mapping includes
/// `FILE_APPEND_DATA`, which is `FILE_CREATE_PIPE_INSTANCE` on a named pipe,
/// and would let a bound client start a competing server instance.
pub const CLIENT_PIPE_RIGHTS: u32 = 0x0012_018B;

/// Pipe DACL: SYSTEM and the current user get full control; each bound SID
/// gets `CLIENT_PIPE_RIGHTS` only. Everyone else is denied by omission.
pub struct PipeSecurity {
    _descriptor: LocalDescriptor,
    attributes: SECURITY_ATTRIBUTES,
}

impl PipeSecurity {
    /// Builds the DACL. Every SID is grammar-checked before it reaches SDDL.
    pub fn new(sids: &[String]) -> io::Result<Self> {
        let current = current_sid()?;
        let mut sddl = format!("D:P(A;;GA;;;SY)(A;;GA;;;{current})");
        for sid in sids {
            check_sid_grammar(sid)?;
            sddl.push_str(&format!("(A;;0x{CLIENT_PIPE_RIGHTS:X};;;{sid})"));
        }
        let descriptor = LocalDescriptor::from_sddl(&sddl)?;
        Ok(Self {
            attributes: descriptor.attributes(),
            _descriptor: descriptor,
        })
    }

    /// Pointer to the `SECURITY_ATTRIBUTES`, valid while `self` is alive.
    pub fn attributes_ptr(&self) -> *mut c_void {
        (&self.attributes as *const SECURITY_ATTRIBUTES)
            .cast_mut()
            .cast()
    }
}

// SAFETY: `attributes` only holds a pointer to the immutable descriptor owned
// by `_descriptor`; see the `LocalDescriptor` impls for why sharing is sound.
unsafe impl Send for PipeSecurity {}
// SAFETY: see the `Send` impl above; shared access is read-only.
unsafe impl Sync for PipeSecurity {}

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
    options
        .first_pipe_instance(first)
        .reject_remote_clients(true);
    // SAFETY: `attributes_ptr` points at a SECURITY_ATTRIBUTES whose descriptor stays alive for the borrow of `security`; tokio passes it to CreateNamedPipeW synchronously and keeps no reference.
    unsafe { options.create_with_security_attributes_raw(name, security.attributes_ptr()) }
}

/// Owns a self-relative security descriptor built from SDDL.
struct LocalDescriptor(PSECURITY_DESCRIPTOR);

// SAFETY: the descriptor is an immutable, LocalAlloc'd self-relative security
// descriptor written once in `from_sddl` and only read afterwards; it is freed
// once, in `drop`. Nothing mutates it through a shared reference, so moving or
// sharing it across threads cannot race.
unsafe impl Send for LocalDescriptor {}
// SAFETY: see the `Send` impl above; shared access is read-only.
unsafe impl Sync for LocalDescriptor {}

impl LocalDescriptor {
    fn from_sddl(sddl: &str) -> io::Result<Self> {
        let wide = to_wide(sddl);
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
        Ok(Self(descriptor))
    }

    /// Attributes naming this descriptor, valid while `self` is alive.
    fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0,
            bInheritHandle: FALSE,
        }
    }
}

impl Drop for LocalDescriptor {
    fn drop(&mut self) {
        // SAFETY: the descriptor came from ConvertStringSecurityDescriptorToSecurityDescriptorW (LocalAlloc) and is freed exactly once.
        unsafe { LocalFree(self.0.cast()) };
    }
}

/// Creates `path` with a protected DACL that grants full control only to the
/// current user and SYSTEM, then writes `bytes` and syncs the file.
///
/// `CREATE_NEW` fails on any existing entry, including a dangling symlink or
/// junction planted at `path`. `FILE_FLAG_OPEN_REPARSE_POINT` also stops the
/// open from following a reparse point in the last path component.
/// Windows analogue of creating a file with mode 0600.
pub fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    let user = current_sid()?;
    check_sid_grammar(&user)?;
    let descriptor = LocalDescriptor::from_sddl(&format!("D:P(A;;FA;;;SY)(A;;FA;;;{user})"))?;
    let attributes = descriptor.attributes();
    let wide = to_wide(path);
    // SAFETY: `wide` is NUL-terminated and outlives the call; `attributes` and the descriptor it points at live until CreateFileW returns and the kernel copies them.
    let raw = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_WRITE,
            0,
            &attributes,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
            null_mut(),
        )
    };
    if raw == INVALID_HANDLE_VALUE {
        return Err(last_error());
    }
    // SAFETY: CreateFileW succeeded, so `raw` is a fresh handle we own.
    let handle = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    let mut file = std::fs::File::from(handle);
    file.write_all(bytes)?;
    file.sync_all()
}

/// Why a pinned directory step was refused.
#[derive(Debug)]
pub enum PinError {
    /// The entry is a symlink or junction. It is never followed.
    ReparsePoint(PathBuf),
    /// The entry exists but is not a directory.
    NotDirectory(PathBuf),
    /// The entry does not exist (only reported as an error by `open`; `entry_state` reports `Missing`).
    Missing(PathBuf),
    /// The resolved path is not a direct child of the pinned parent.
    Escaped(PathBuf),
    /// Any other OS failure.
    Io(io::Error),
}

/// What an entry under a pinned directory is. Read from an open handle that
/// does not follow reparse points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryState {
    Missing,
    File,
    Directory,
    ReparsePoint,
}

/// A directory opened for a containment-checked write.
///
/// The handle is opened without `FILE_SHARE_DELETE`, so while it is held no
/// other handle can rename, delete, or replace this directory. The path
/// recorded at open time therefore keeps naming the same directory for the
/// lifetime of the handle. Every step below the pinned directory is opened
/// with `FILE_FLAG_OPEN_REPARSE_POINT`, refused if it is a reparse point, and
/// checked to be a direct child of its pinned parent.
pub struct PinnedDir {
    handle: OwnedHandle,
    path: PathBuf,
}

/// Monotonic counter so temp-sibling names do not collide within a process.
static TEMP_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

const PINNED_DIR_ACCESS: u32 = GENERIC_READ | GENERIC_WRITE;
const DIRECTORY_FLAGS: u32 = FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT;

fn is_not_found(error: &io::Error) -> bool {
    matches!(
        error.raw_os_error(),
        Some(code) if code == ERROR_FILE_NOT_FOUND as i32 || code == ERROR_PATH_NOT_FOUND as i32
    )
}

fn handle_attributes(handle: HANDLE) -> io::Result<u32> {
    // SAFETY: `info` is a plain out-struct of the layout the call writes; the handle is live and owned by the caller.
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { mem::zeroed() };
    // SAFETY: the handle is live and `info` is a valid out-pointer.
    if unsafe { GetFileInformationByHandle(handle, &mut info) } == FALSE {
        return Err(last_error());
    }
    Ok(info.dwFileAttributes)
}

fn handle_final_path(handle: HANDLE) -> io::Result<PathBuf> {
    let mut buffer = vec![0u16; 1024];
    loop {
        // SAFETY: `buffer` is writable for `buffer.len()` UTF-16 units and the handle is live.
        let needed = unsafe {
            GetFinalPathNameByHandleW(
                handle,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                VOLUME_NAME_DOS,
            )
        } as usize;
        if needed == 0 {
            return Err(last_error());
        }
        if needed < buffer.len() {
            buffer.truncate(needed);
            return Ok(PathBuf::from(std::ffi::OsString::from_wide(&buffer)));
        }
        // The buffer was too small: `needed` is the size including the NUL.
        buffer.resize(needed + 1, 0);
    }
}

/// Opens `path` with the given access, share mode, disposition and flags.
fn open_entry(
    path: &Path,
    access: u32,
    share: u32,
    disposition: u32,
    flags: u32,
) -> io::Result<OwnedHandle> {
    let wide = to_wide(path);
    // SAFETY: `wide` is NUL-terminated and outlives the call; null security attributes and template handle are valid.
    let raw = unsafe {
        CreateFileW(
            wide.as_ptr(),
            access,
            share,
            null(),
            disposition,
            flags,
            null_mut(),
        )
    };
    if raw == INVALID_HANDLE_VALUE {
        return Err(last_error());
    }
    // SAFETY: CreateFileW succeeded, so `raw` is a fresh handle we own.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw as _) })
}

impl PinnedDir {
    /// Opens `path` as a pinned directory. Refuses a reparse point and a
    /// non-directory. The returned path is the handle's resolved path.
    pub fn open(path: &Path) -> Result<PinnedDir, PinError> {
        let handle = open_entry(
            path,
            PINNED_DIR_ACCESS,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            OPEN_EXISTING,
            DIRECTORY_FLAGS,
        )
        .map_err(|error| {
            if is_not_found(&error) {
                PinError::Missing(path.to_path_buf())
            } else {
                PinError::Io(error)
            }
        })?;
        let attributes =
            handle_attributes(handle.as_raw_handle() as HANDLE).map_err(PinError::Io)?;
        if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(PinError::ReparsePoint(path.to_path_buf()));
        }
        if attributes & FILE_ATTRIBUTE_DIRECTORY == 0 {
            return Err(PinError::NotDirectory(path.to_path_buf()));
        }
        let resolved = handle_final_path(handle.as_raw_handle() as HANDLE).map_err(PinError::Io)?;
        Ok(PinnedDir {
            handle,
            path: resolved,
        })
    }

    /// The resolved path of the pinned directory (`VOLUME_NAME_DOS`).
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Opens the direct child `name` of this directory as a pinned directory.
    /// With `create`, a missing child is created first, under this pinned
    /// parent only. Nothing is created outside this directory.
    pub fn child_dir(&self, name: &OsStr, create: bool) -> Result<PinnedDir, PinError> {
        let child = self.path.join(name);
        if create {
            let wide = to_wide(&child);
            // SAFETY: `wide` is NUL-terminated and outlives the call; null security attributes are valid.
            if unsafe { CreateDirectoryW(wide.as_ptr(), null()) } == FALSE {
                let error = last_error();
                if error.raw_os_error() != Some(ERROR_ALREADY_EXISTS as i32) {
                    return Err(PinError::Io(error));
                }
            }
        }
        let dir = PinnedDir::open(&child)?;
        if dir.path.parent() != Some(self.path.as_path()) {
            return Err(PinError::Escaped(dir.path));
        }
        Ok(dir)
    }

    /// Reports what `name` is, without following a reparse point.
    pub fn entry_state(&self, name: &OsStr) -> Result<EntryState, PinError> {
        let child = self.path.join(name);
        let handle = match open_entry(
            &child,
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            OPEN_EXISTING,
            DIRECTORY_FLAGS,
        ) {
            Ok(handle) => handle,
            Err(error) if is_not_found(&error) => return Ok(EntryState::Missing),
            Err(error) => return Err(PinError::Io(error)),
        };
        let attributes =
            handle_attributes(handle.as_raw_handle() as HANDLE).map_err(PinError::Io)?;
        Ok(if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            EntryState::ReparsePoint
        } else if attributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            EntryState::Directory
        } else {
            EntryState::File
        })
    }

    /// Replaces the entry `name` in this directory with `bytes`, without
    /// touching any existing inode.
    ///
    /// The bytes go to a fresh temp sibling created with `CREATE_NEW`, which
    /// is flushed and then renamed onto `name` with `ReplaceIfExists`, using
    /// this pinned directory as the rename root. An existing hard link to the
    /// old entry keeps the old bytes: only the directory entry changes. On
    /// failure the temp sibling is marked delete-on-close.
    pub fn write_replacing(&self, name: &OsStr, bytes: &[u8]) -> io::Result<()> {
        use std::io::Write;
        use std::sync::atomic::Ordering;
        let mut attempts = 0;
        let handle = loop {
            let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let mut temp_name = std::ffi::OsString::from(".");
            temp_name.push(name);
            temp_name.push(format!(".sf-tmp-{}-{sequence}", std::process::id()));
            let temp_path = self.path.join(&temp_name);
            match open_entry(
                &temp_path,
                GENERIC_WRITE | DELETE,
                FILE_SHARE_READ,
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
            ) {
                Ok(handle) => break handle,
                Err(error)
                    if error.raw_os_error() == Some(ERROR_FILE_EXISTS as i32) && attempts < 16 =>
                {
                    attempts += 1;
                }
                Err(error) => return Err(error),
            }
        };
        let mut file = std::fs::File::from(handle);
        let outcome = file
            .write_all(bytes)
            .and_then(|()| file.sync_all())
            .and_then(|()| {
                rename_handle_into(
                    file.as_raw_handle() as HANDLE,
                    self.handle.as_raw_handle() as HANDLE,
                    name,
                )
            });
        if outcome.is_err() {
            let info = FILE_DISPOSITION_INFO { DeleteFile: true };
            // SAFETY: the handle is live and was opened with DELETE access; `info` is a live struct of the size passed.
            unsafe {
                SetFileInformationByHandle(
                    file.as_raw_handle() as HANDLE,
                    FileDispositionInfo,
                    (&info as *const FILE_DISPOSITION_INFO).cast(),
                    mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
                )
            };
        }
        outcome
    }
}

/// Renames the open file `file` to `name` inside the directory `root_dir`,
/// replacing an existing entry. `name` must be a single relative component.
fn rename_handle_into(file: HANDLE, root_dir: HANDLE, name: &OsStr) -> io::Result<()> {
    let name_wide: Vec<u16> = name.encode_wide().collect();
    let name_bytes = name_wide.len() * mem::size_of::<u16>();
    // FILE_RENAME_INFO already holds one UTF-16 unit in `FileName`, so the
    // header plus `name_bytes` is always large enough for the name.
    let size = mem::size_of::<FILE_RENAME_INFO>() + name_bytes;
    // u64 storage keeps the buffer aligned for the FILE_RENAME_INFO header.
    let mut buffer = vec![0u64; size.div_ceil(mem::size_of::<u64>())];
    let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    // SAFETY: `buffer` is zeroed, 8-byte aligned and at least `size` bytes, so the header and `name_bytes` of FileName are in bounds and initialized. `FileName` starts at a u16-aligned offset inside the header allocation.
    unsafe {
        (*info).Anonymous = FILE_RENAME_INFO_0 {
            ReplaceIfExists: true,
        };
        (*info).RootDirectory = root_dir;
        (*info).FileNameLength = name_bytes as u32;
        let destination = std::ptr::addr_of_mut!((*info).FileName).cast::<u16>();
        std::ptr::copy_nonoverlapping(name_wide.as_ptr(), destination, name_wide.len());
    }
    // SAFETY: `file` is a live handle opened with DELETE access; `info` points at `size` initialized bytes that outlive the call.
    let ok = unsafe { SetFileInformationByHandle(file, FileRenameInfo, info.cast(), size as u32) };
    if ok == FALSE {
        return Err(last_error());
    }
    Ok(())
}

/// Resumes every thread of `pid`. Fails if none was resumed.
fn resume_threads(pid: u32) -> io::Result<()> {
    // SAFETY: a thread snapshot takes no pointers; the handle is checked before it is owned.
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if raw == INVALID_HANDLE_VALUE {
        return Err(last_error());
    }
    // SAFETY: CreateToolhelp32Snapshot succeeded, so `raw` is a fresh handle we own.
    let snapshot = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    // SAFETY: THREADENTRY32 is plain data; all-zero is a valid starting value and dwSize is set below.
    let mut entry: THREADENTRY32 = unsafe { mem::zeroed() };
    entry.dwSize = mem::size_of::<THREADENTRY32>() as u32;
    // SAFETY: `snapshot` is live and `entry` is a live struct with dwSize set as the API requires.
    let mut more =
        unsafe { Thread32First(snapshot.as_raw_handle() as HANDLE, &mut entry) } != FALSE;
    let mut resumed = 0u32;
    while more {
        if entry.th32OwnerProcessID == pid {
            // SAFETY: plain call with no pointers; a null return is checked below.
            let raw = unsafe { OpenThread(THREAD_SUSPEND_RESUME, FALSE, entry.th32ThreadID) };
            if raw.is_null() {
                return Err(last_error());
            }
            // SAFETY: OpenThread returned a fresh, non-null handle we own.
            let thread = unsafe { OwnedHandle::from_raw_handle(raw as _) };
            // SAFETY: the thread handle is live; ResumeThread returns u32::MAX on failure.
            if unsafe { ResumeThread(thread.as_raw_handle() as HANDLE) } == u32::MAX {
                return Err(last_error());
            }
            resumed += 1;
        }
        // SAFETY: `snapshot` and `entry` are the live objects used by the enumeration above.
        more = unsafe { Thread32Next(snapshot.as_raw_handle() as HANDLE, &mut entry) } != FALSE;
    }
    if resumed == 0 {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no thread of the new process was found to resume",
        ));
    }
    Ok(())
}

/// Spawns `cmd` suspended, assigns it to a new kill-on-close job, then resumes
/// its threads. The primary thread never runs before the assignment, so a
/// grandchild cannot be created outside the job.
///
/// `CREATE_SUSPENDED` is set with `creation_flags`, which replaces any flags
/// set earlier on `cmd`: std does not expose them for merging. Callers in this
/// workspace set no other creation flags. On any failure after the spawn the
/// process tree is terminated and the error is returned.
pub fn spawn_in_job(cmd: &mut std::process::Command) -> io::Result<(Child, Job)> {
    use std::os::windows::process::CommandExt;
    let job = Job::kill_on_close()?;
    cmd.creation_flags(CREATE_SUSPENDED);
    let mut child = cmd.spawn()?;
    if let Err(error) = job.assign(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    if let Err(error) = resume_threads(child.id()) {
        let _ = job.terminate();
        let _ = child.wait();
        return Err(error);
    }
    Ok((child, job))
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

    /// Terminates every process in the job, then waits until the job is empty
    /// or `deadline` passes. Termination is asynchronous, so a count read
    /// right after `terminate` can still be non-zero. Survivors are reported
    /// only after the deadline.
    pub fn terminate_and_wait(&self, deadline: Duration) -> io::Result<()> {
        self.terminate()?;
        let until = Instant::now() + deadline;
        loop {
            let remaining = self.active_processes()?;
            if remaining == 0 {
                return Ok(());
            }
            if Instant::now() >= until {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("{remaining} job process(es) survived termination"),
                ));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
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
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use std::process::{Command, Stdio};
    use windows_sys::Win32::Security::Authorization::{
        ConvertSecurityDescriptorToStringSecurityDescriptorW, GetNamedSecurityInfoW, SE_FILE_OBJECT,
    };
    use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;

    #[test]
    fn current_sid_is_a_user_sid() {
        let sid = current_sid().expect("current sid");
        assert!(sid.starts_with("S-1-5-"), "{sid}");
        check_sid_grammar(&sid).expect("grammar");
    }

    #[test]
    fn pipe_security_rejects_malformed_sids() {
        for bad in [
            "",
            "S-1-",
            "S-1-5-21-x",
            "S-1-5-21-1)(A;;GA;;;WD",
            "BU",
            "S-1-5--21",
            "S-1-5-21 ",
        ] {
            let err = PipeSecurity::new(&[bad.to_string()]).err();
            assert!(err.is_some(), "accepted {bad:?}");
        }
    }

    #[test]
    fn pipe_security_accepts_well_formed_sid() {
        let sid = current_sid().expect("current sid");
        PipeSecurity::new(&[sid]).expect("well-formed sid builds a descriptor");
    }

    /// Pids whose parent is `parent`, from a process snapshot.
    fn child_processes(parent: u32) -> Vec<u32> {
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
        };
        // SAFETY: a process snapshot takes no pointers; the handle is checked before it is owned.
        let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        assert_ne!(raw, INVALID_HANDLE_VALUE, "process snapshot");
        // SAFETY: the snapshot succeeded, so `raw` is a fresh handle we own.
        let snapshot = unsafe { OwnedHandle::from_raw_handle(raw as _) };
        // SAFETY: PROCESSENTRY32W is plain data; all-zero is valid and dwSize is set below.
        let mut entry: PROCESSENTRY32W = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;
        // SAFETY: `snapshot` and `entry` are live and match the enumeration API.
        let mut more =
            unsafe { Process32FirstW(snapshot.as_raw_handle() as HANDLE, &mut entry) } != FALSE;
        let mut found = Vec::new();
        while more {
            if entry.th32ParentProcessID == parent {
                found.push(entry.th32ProcessID);
            }
            // SAFETY: same live objects as the first call.
            more =
                unsafe { Process32NextW(snapshot.as_raw_handle() as HANDLE, &mut entry) } != FALSE;
        }
        found
    }

    /// The SDDL text of a file's DACL, read back from the kernel.
    fn dacl_sddl(path: &std::path::Path) -> String {
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut sd: PSECURITY_DESCRIPTOR = null_mut();
        // SAFETY: `wide` is NUL-terminated; only the descriptor out-pointer is non-null; the kernel allocates it with LocalAlloc.
        let status = unsafe {
            GetNamedSecurityInfoW(
                wide.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                &mut sd,
            )
        };
        assert_eq!(status, 0, "GetNamedSecurityInfoW");
        let descriptor = LocalDescriptor(sd);
        let mut out: *mut u16 = null_mut();
        // SAFETY: `descriptor` is a live descriptor; `out` receives a LocalAlloc'd string freed below.
        let ok = unsafe {
            ConvertSecurityDescriptorToStringSecurityDescriptorW(
                descriptor.0,
                SDDL_REVISION_1,
                DACL_SECURITY_INFORMATION,
                &mut out,
                null_mut(),
            )
        };
        assert_ne!(
            ok, FALSE,
            "ConvertSecurityDescriptorToStringSecurityDescriptorW"
        );
        let text = wide_ptr_to_string(out).expect("sddl text");
        // SAFETY: `out` was allocated by the conversion above and is freed exactly once.
        unsafe { LocalFree(out.cast()) };
        text
    }

    #[test]
    fn write_private_creates_a_protected_two_ace_dacl() {
        let dir = std::env::temp_dir().join(format!("sf-os-private-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("key.bin");
        let _ = std::fs::remove_file(&path);
        write_private(&path, b"secret").expect("write_private");
        assert_eq!(std::fs::read(&path).expect("read back"), b"secret");
        let sddl = dacl_sddl(&path);
        let sid = current_sid().expect("sid");
        assert!(sddl.starts_with("D:P("), "DACL is not protected: {sddl}");
        assert_eq!(sddl.matches("(A;").count(), 2, "{sddl}");
        assert!(sddl.contains("(A;;FA;;;SY)"), "{sddl}");
        // SDDL renders well-known SIDs as aliases. The runner is the local
        // Administrator (RID 500), which the conversion prints as `LA`.
        let others: Vec<&str> = sddl
            .split('(')
            .filter_map(|ace| ace.strip_prefix("A;;FA;;;"))
            .map(|sid| sid.trim_end_matches(')'))
            .filter(|sid| *sid != "SY")
            .collect();
        assert_eq!(others.len(), 1, "{sddl}");
        let alias_ok = others[0] == "LA" && sid.ends_with("-500");
        assert!(others[0] == sid || alias_ok, "{sddl}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_private_refuses_existing_entries_and_reparse_points() {
        let dir = std::env::temp_dir().join(format!("sf-os-refuse-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let existing = dir.join("existing.bin");
        std::fs::write(&existing, b"old").expect("seed");
        let err = write_private(&existing, b"new").expect_err("existing file");
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&existing).expect("read"), b"old");

        let planted = dir.join("planted.bin");
        let target = dir.join("target.bin");
        std::fs::write(&target, b"target").expect("target");
        match std::os::windows::fs::symlink_file(&target, &planted) {
            Ok(()) => {
                assert!(write_private(&planted, b"new").is_err(), "symlink followed");
                assert_eq!(std::fs::read(&target).expect("read"), b"target");
            }
            Err(error) => {
                let reason = format!("symlink creation needs privilege ({error})");
                if std::env::var("SEA_FORGE_REQUIRE_LINK_PRIVILEGE").as_deref() == Ok("1") {
                    panic!("BLOCKED prerequisite, not a pass: {reason}");
                }
                println!("SKIPPED: {reason}");
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pinned_dir_replaces_a_hard_linked_entry_without_touching_the_link() {
        let dir = std::env::temp_dir().join(format!("sf-os-pinned-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dir");
        let outside = dir.join("outside.txt");
        std::fs::write(&outside, b"keep").expect("seed");
        let linked = dir.join("linked.txt");
        std::fs::hard_link(&outside, &linked).expect("hard link");
        let pinned = PinnedDir::open(&dir).expect("open pinned");
        assert_eq!(
            pinned.entry_state(OsStr::new("linked.txt")).expect("state"),
            EntryState::File
        );
        pinned
            .write_replacing(OsStr::new("linked.txt"), b"new")
            .expect("replace");
        assert_eq!(std::fs::read(&outside).expect("read"), b"keep");
        assert_eq!(std::fs::read(&linked).expect("read"), b"new");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn terminate_and_wait_returns_once_the_job_is_empty() {
        let job = Job::kill_on_close().expect("job");
        let mut child = Command::new("cmd")
            .args(["/c", "ping", "-n", "30", "127.0.0.1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn");
        job.assign(&child).expect("assign");
        job.terminate_and_wait(Duration::from_secs(5))
            .expect("job emptied within the deadline");
        assert_eq!(job.active_processes().expect("count"), 0);
        let _ = child.wait();
    }

    #[test]
    fn spawn_in_job_contains_a_grandchild_created_at_startup() {
        let mut command = Command::new("cmd");
        command
            .args(["/c", "start /b ping -n 30 127.0.0.1 & ping -n 30 127.0.0.1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let (mut child, job) = spawn_in_job(&mut command).expect("spawn in job");
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut grandchildren = Vec::new();
        while grandchildren.is_empty() && Instant::now() < deadline {
            grandchildren = child_processes(child.id());
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(
            !grandchildren.is_empty(),
            "no grandchild observed under the child"
        );
        assert!(
            job.active_processes().expect("count") >= 2,
            "grandchild is outside the job"
        );
        job.terminate().expect("terminate");
        let _ = child.wait();
        let deadline = Instant::now() + Duration::from_secs(10);
        while job.active_processes().expect("count") != 0 {
            assert!(
                Instant::now() < deadline,
                "job processes survived termination"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
        for pid in grandchildren {
            let listing = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
                .output()
                .expect("tasklist");
            let text = String::from_utf8_lossy(&listing.stdout).to_lowercase();
            assert!(!text.contains("ping.exe"), "ping {pid} survived: {text}");
        }
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
