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

use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
use windows_sys::Wdk::Storage::FileSystem::{
    NtCreateFile, FILE_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_FOR_BACKUP_INTENT, FILE_OPEN_IF,
};
use windows_sys::Win32::Foundation::{
    CloseHandle, LocalFree, RtlNtStatusToDosError, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND,
    ERROR_SHARING_VIOLATION, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, NTSTATUS,
    OBJ_CASE_INSENSITIVE, UNICODE_STRING,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, RevertToSelf, SecurityIdentification, TokenImpersonationLevel, TokenUser,
    PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FileAttributeTagInfo, FileDispositionInfo, FileIdInfo, GetFileInformationByHandle,
    GetFileInformationByHandleEx, GetFinalPathNameByHandleW, GetLongPathNameW,
    SetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, CREATE_NEW, DELETE,
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_ATTRIBUTE_TAG_INFO, FILE_DISPOSITION_INFO, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_ID_INFO, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE,
    FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, VOLUME_NAME_DOS,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob,
    JobObjectBasicAccountingInformation, JobObjectExtendedLimitInformation,
    QueryInformationJobObject, SetInformationJobObject, TerminateJobObject,
    JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::Pipes::{GetNamedPipeServerProcessId, ImpersonateNamedPipeClient};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, OpenProcess, OpenProcessToken, OpenThread,
    OpenThreadToken, ResumeThread, CREATE_SUSPENDED, PROCESS_QUERY_LIMITED_INFORMATION,
    THREAD_SUSPEND_RESUME,
};
use windows_sys::Win32::System::IO::IO_STATUS_BLOCK;

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
    /// Another handle already holds the directory for write or delete access,
    /// so the exclusive pin cannot be taken. Reported instead of degrading to
    /// a shared pin: an untracked writer could mutate the directory mid-write.
    WriteShareHeld(PathBuf),
    /// The pinned object's identity (volume serial + file id) changed since
    /// the pin was taken or since the previous use, so the handle no longer
    /// names the validated directory.
    IdentityChanged(PathBuf),
    /// The opened handle's final path does not name the path that was asked
    /// for, so the handle resolved somewhere else than the caller named.
    FinalPathMismatch(PathBuf),
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
/// The pin is exclusive: the handle is opened with share mode
/// `FILE_SHARE_READ` only. Sharing violations therefore guard both sides of
/// the pin. After `open` succeeds, any later open of this directory that
/// requests write or delete access (including entry creation, renames, and
/// junction planting by path) fails while the pin is held. And if a handle
/// that already holds the directory for write or delete access exists when
/// the pin is taken, `open` itself fails with [`PinError::WriteShareHeld`]
/// instead of pinning alongside an untracked mutator.
///
/// Legitimate writers: only SEA-Forge's materialization writes go through a
/// `PinnedDir`. Workload file writes open files, not pinned directories, and
/// the sandbox's own child operations below the pin are handle-anchored
/// (`RootDirectory`-relative `NtCreateFile`), so none of them needs a second
/// write-class open of the pinned directory — the publish rename in
/// `write_replacing` is the one operation that does, and it releases the pin
/// for exactly that call (see its invariant note). A workload `FILE_ADD_FILE`
/// open of a pinned directory during the pin window would transiently fail
/// with a sharing violation; that window is bounded to a single
/// materialization write (the pin ends when `write_replacing` returns) and
/// does not overlap workload execution.
///
/// The handle is also opened without `FILE_SHARE_DELETE`, so while it is held
/// no other handle can rename, delete, or replace this directory. The path
/// recorded at open time therefore keeps naming the same directory for the
/// lifetime of the handle. Every step below the pinned directory is opened
/// anchored to this handle with `FILE_OPEN_REPARSE_POINT`, refused if it is a
/// reparse point, and checked to be a direct child of its pinned parent; the
/// pin's reparse tag and `FILE_ID_INFO` identity are re-checked before every
/// child operation, so a mutation that slips past the share mode is still
/// refused at use.
pub struct PinnedDir {
    /// The pinned directory handle: anchor for child operations and subject
    /// of the per-use revalidation.
    handle: OwnedHandle,
    path: PathBuf,
    /// Identity (`FILE_ID_INFO`) recorded at open time and re-checked per use.
    id: FILE_ID_INFO,
}

/// Monotonic counter so temp-sibling names do not collide within a process.
static TEMP_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

const PINNED_DIR_ACCESS: u32 = GENERIC_READ | GENERIC_WRITE;
const DIRECTORY_FLAGS: u32 = FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT;
/// The pin is exclusive: share read access only, so no other handle can open
/// the pinned directory for write or delete access while the pin is held.
/// (`FILE_SHARE_WRITE` would admit exactly the concurrent directory mutator
/// this pin exists to exclude; `FILE_SHARE_DELETE` would admit renames.)
const PINNED_DIR_SHARE: u32 = FILE_SHARE_READ;

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

/// Maps an `ntdll` status code to the equivalent Win32 error.
fn nt_error(status: NTSTATUS) -> io::Error {
    // SAFETY: plain call with a status value; the mapping table is read-only.
    let code = unsafe { RtlNtStatusToDosError(status) };
    io::Error::from_raw_os_error(code as i32)
}

/// `FILE_CREATE` against an existing entry (`STATUS_OBJECT_NAME_COLLISION`
/// Reparse tag and attribute word of an open handle (`FileAttributeTagInfo`).
fn handle_attribute_tag(handle: HANDLE) -> io::Result<FILE_ATTRIBUTE_TAG_INFO> {
    // SAFETY: FILE_ATTRIBUTE_TAG_INFO is plain data; all-zero is a valid initial value.
    let mut info: FILE_ATTRIBUTE_TAG_INFO = unsafe { mem::zeroed() };
    // SAFETY: `info` is a live out-buffer of the exact size the class writes; the handle is live and owned by the caller.
    let ok = unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileAttributeTagInfo,
            (&mut info as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    };
    if ok == FALSE {
        return Err(last_error());
    }
    Ok(info)
}

/// Volume serial number and file id of an open handle (`FileIdInfo`): the
/// Windows dev/ino pair used to re-confirm object identity at use.
fn handle_id(handle: HANDLE) -> io::Result<FILE_ID_INFO> {
    // SAFETY: FILE_ID_INFO is plain data; all-zero is a valid initial value.
    let mut info: FILE_ID_INFO = unsafe { mem::zeroed() };
    // SAFETY: `info` is a live out-buffer of the exact size the class writes; the handle is live and owned by the caller.
    let ok = unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileIdInfo,
            (&mut info as *mut FILE_ID_INFO).cast(),
            mem::size_of::<FILE_ID_INFO>() as u32,
        )
    };
    if ok == FALSE {
        return Err(last_error());
    }
    Ok(info)
}

/// Refuses a reparse point or a non-directory on an open handle, reporting
/// `path` in the error.
fn refuse_reparse_or_file(handle: HANDLE, path: &Path) -> Result<(), PinError> {
    let info = handle_attribute_tag(handle).map_err(PinError::Io)?;
    if info.ReparseTag != 0 || info.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(PinError::ReparsePoint(path.to_path_buf()));
    }
    if info.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0 {
        return Err(PinError::NotDirectory(path.to_path_buf()));
    }
    Ok(())
}

/// Strips the `\\?\` / `\\?\UNC\` device prefix that
/// `GetFinalPathNameByHandleW(VOLUME_NAME_DOS)` always emits, so a final path
/// can be compared with a plain `GetLongPathNameW` result.
fn strip_device_prefix(path: &Path) -> String {
    let text = path.as_os_str().to_string_lossy().into_owned();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = text.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        text
    }
}

/// Case-insensitive, separator-folded path comparison: Windows path names are
/// case-insensitive and `/` and `\` are equivalent input separators.
fn same_path(a: &str, b: &str) -> bool {
    let fold = |text: &str| text.to_lowercase().replace('/', "\\");
    fold(a) == fold(b)
}

/// The long (8.3-expanded) form of `path`, as `GetLongPathNameW` reports it.
fn long_path(path: &Path) -> io::Result<PathBuf> {
    let wide = to_wide(path);
    let mut buffer = vec![0u16; 1024];
    loop {
        // SAFETY: `wide` is NUL-terminated and outlives the call; `buffer` is writable for its full length.
        let needed =
            unsafe { GetLongPathNameW(wide.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32) };
        if needed == 0 {
            return Err(last_error());
        }
        if (needed as usize) < buffer.len() {
            buffer.truncate(needed as usize);
            return Ok(PathBuf::from(std::ffi::OsString::from_wide(&buffer)));
        }
        // The buffer was too small: `needed` is the size including the NUL.
        buffer.resize(needed as usize + 1, 0);
    }
}

/// Confirms the handle's final path names the path that was asked for
/// (`GetFinalPathNameByHandleW` == `GetLongPathNameW`): a name that resolves
/// through a planted reparse point or an alias is refused instead of pinned
/// under a different object than the caller named.
fn confirm_final_path(handle: HANDLE, requested: &Path) -> Result<(), PinError> {
    let final_path = handle_final_path(handle).map_err(PinError::Io)?;
    // `GetLongPathNameW` rejects some verbatim forms, so the device prefix is
    // stripped before the expansion and the comparison runs on plain paths.
    let requested_text = strip_device_prefix(requested);
    let long = match long_path(Path::new(&requested_text)) {
        Ok(long) => strip_device_prefix(&long),
        // If the expansion itself fails, comparing against the unexpanded
        // name can only produce a false refusal, never a false confirmation.
        Err(_) => requested_text.clone(),
    };
    if !same_path(&strip_device_prefix(&final_path), &long) {
        return Err(PinError::FinalPathMismatch(requested.to_path_buf()));
    }
    Ok(())
}

/// Opens the single-component `name` inside the directory `parent` with the
/// object manager resolving the name against `parent` itself
/// (`OBJECT_ATTRIBUTES::RootDirectory`), so the final component is never
/// re-resolved by pathname: no swap of a directory entry can redirect the
/// open after the parent was validated. The caller passes
/// `FILE_FLAG_OPEN_REPARSE_POINT` (equal to the NT create option
/// `FILE_OPEN_REPARSE_POINT`) in `options` so a reparse point in the final
/// component is opened as itself and refused by the caller's tag check
/// instead of being traversed.
fn open_anchored(
    parent: HANDLE,
    name: &OsStr,
    access: u32,
    share: u32,
    disposition: u32,
    options: u32,
) -> io::Result<OwnedHandle> {
    let name_wide: Vec<u16> = name.encode_wide().collect();
    let name_bytes = name_wide.len() * mem::size_of::<u16>();
    // `UNICODE_STRING::Length` is counted in BYTES (not characters) and
    // excludes the terminator; a component name that cannot fit a u16 byte
    // count is refused instead of truncated.
    if name_wide.is_empty() || name_bytes > (u16::MAX - 2) as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "anchored name must be a non-empty single component",
        ));
    }
    let mut name = UNICODE_STRING {
        Length: name_bytes as u16,
        MaximumLength: (name_bytes + 2) as u16,
        Buffer: name_wide.as_ptr() as *mut u16,
    };
    let attributes = OBJECT_ATTRIBUTES {
        Length: mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: parent,
        ObjectName: &mut name,
        Attributes: OBJ_CASE_INSENSITIVE,
        SecurityDescriptor: null(),
        SecurityQualityOfService: null(),
    };
    let mut raw: HANDLE = null_mut();
    // SAFETY: IO_STATUS_BLOCK is plain data; all-zero is a valid initial value.
    let mut status_block: IO_STATUS_BLOCK = unsafe { mem::zeroed() };
    // SAFETY: `name_wide` outlives the call and `name` points into it; `attributes` only references them and the live `parent` handle; `raw` and `status_block` are valid out-objects.
    let status = unsafe {
        NtCreateFile(
            &mut raw,
            access,
            &attributes,
            &mut status_block,
            null(),
            FILE_ATTRIBUTE_NORMAL,
            share,
            disposition,
            options,
            null(),
            0,
        )
    };
    if status < 0 {
        return Err(nt_error(status));
    }
    // SAFETY: NtCreateFile returned a success status, so `raw` is a fresh handle we own.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw as _) })
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
    /// Opens `path` as a pinned directory with an exclusive pin. Refuses a
    /// reparse point, a non-directory, a directory another handle already
    /// holds for write or delete access ([`PinError::WriteShareHeld`], loud,
    /// never degraded to a shared pin), and a name that resolves to a
    /// different object than it names ([`PinError::FinalPathMismatch`]). The
    /// returned path is the handle's resolved path.
    pub fn open(path: &Path) -> Result<PinnedDir, PinError> {
        let handle = open_entry(
            path,
            PINNED_DIR_ACCESS,
            PINNED_DIR_SHARE,
            OPEN_EXISTING,
            DIRECTORY_FLAGS,
        )
        .map_err(|error| {
            if is_not_found(&error) {
                PinError::Missing(path.to_path_buf())
            } else if error.raw_os_error() == Some(ERROR_SHARING_VIOLATION as i32) {
                PinError::WriteShareHeld(path.to_path_buf())
            } else {
                PinError::Io(error)
            }
        })?;
        let raw = handle.as_raw_handle() as HANDLE;
        refuse_reparse_or_file(raw, path)?;
        confirm_final_path(raw, path)?;
        let id = handle_id(raw).map_err(PinError::Io)?;
        let resolved = handle_final_path(raw).map_err(PinError::Io)?;
        Ok(PinnedDir {
            handle,
            path: resolved,
            id,
        })
    }

    /// The resolved path of the pinned directory (`VOLUME_NAME_DOS`).
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Ends the exclusive pin, consuming the pinned directory.
    ///
    /// Closing the handle is the release: while the handle exists no other
    /// opener can take a write-class open of this directory (the pin's
    /// `FILE_SHARE_READ`-only share refuses them), and the kernel's
    /// replace-rename needs exactly such an open of the target's parent.
    /// [`PinnedDir::write_replacing`] releases the pin around its rename for
    /// that reason and re-opens a fresh pin immediately afterwards. A close
    /// failure is reported instead of being silently dropped.
    pub fn release(self) -> io::Result<()> {
        // SAFETY: `raw` is owned by the consumed `PinnedDir`'s handle and is
        // closed exactly once here; `forget` prevents the second close that
        // `OwnedHandle`'s drop would perform.
        let raw = self.handle.as_raw_handle();
        let ok = unsafe { CloseHandle(raw as _) };
        let result = if ok == FALSE {
            Err(last_error())
        } else {
            Ok(())
        };
        std::mem::forget(self.handle);
        result
    }

    /// Re-checks the pinned handle before a child operation: the object must
    /// still be a non-reparse directory and must still be the object whose
    /// `FILE_ID_INFO` identity was recorded at open. This is the backstop
    /// that catches a mutation that slipped past the exclusive share mode
    /// (for example a writer handle that predated the pin and set a reparse
    /// point on the directory mid-operation).
    fn revalidate(&self) -> Result<(), PinError> {
        let raw = self.handle.as_raw_handle() as HANDLE;
        refuse_reparse_or_file(raw, &self.path)?;
        let id = handle_id(raw).map_err(PinError::Io)?;
        if id.VolumeSerialNumber != self.id.VolumeSerialNumber
            || id.FileId.Identifier != self.id.FileId.Identifier
        {
            return Err(PinError::IdentityChanged(self.path.clone()));
        }
        Ok(())
    }

    /// Opens the direct child `name` of this directory as a pinned directory.
    /// With `create`, a missing child is created first, anchored to this
    /// pinned parent only; nothing is created outside this directory.
    pub fn child_dir(&self, name: &OsStr, create: bool) -> Result<PinnedDir, PinError> {
        self.revalidate()?;
        // `FILE_DIRECTORY_FILE` is required on the create branch (otherwise a
        // missing name would be created as a file) and omitted on the open
        // branch so an existing plain file is reported as `NotDirectory` by
        // the tag check below instead of at the open. The open branch does
        // carry `FILE_OPEN_FOR_BACKUP_INTENT` (the NT create option behind
        // `FILE_FLAG_BACKUP_SEMANTICS`): without it or `FILE_DIRECTORY_FILE`
        // `NtCreateFile` refuses an existing directory with
        // `STATUS_FILE_IS_A_DIRECTORY`, making every existing child dir
        // unopenable. `FILE_FLAG_OPEN_REPARSE_POINT` (equal to the NT create
        // option `FILE_OPEN_REPARSE_POINT`) opens a junction as itself so
        // the tag check refuses it instead of traversing it.
        let options = if create {
            FILE_DIRECTORY_FILE | FILE_FLAG_OPEN_REPARSE_POINT
        } else {
            FILE_OPEN_FOR_BACKUP_INTENT | FILE_FLAG_OPEN_REPARSE_POINT
        };
        let handle = open_anchored(
            self.handle.as_raw_handle() as HANDLE,
            name,
            PINNED_DIR_ACCESS,
            PINNED_DIR_SHARE,
            if create { FILE_OPEN_IF } else { FILE_OPEN },
            options,
        )
        .map_err(|error| {
            if is_not_found(&error) {
                PinError::Missing(self.path.join(name))
            } else {
                PinError::Io(error)
            }
        })?;
        let raw = handle.as_raw_handle() as HANDLE;
        let child_path = self.path.join(name);
        refuse_reparse_or_file(raw, &child_path)?;
        let id = handle_id(raw).map_err(PinError::Io)?;
        let resolved = handle_final_path(raw).map_err(PinError::Io)?;
        if resolved.parent() != Some(self.path.as_path()) {
            return Err(PinError::Escaped(resolved));
        }
        Ok(PinnedDir {
            handle,
            path: resolved,
            id,
        })
    }

    /// Reports what `name` is, without following a reparse point. The entry
    /// is opened anchored to this pinned handle.
    pub fn entry_state(&self, name: &OsStr) -> Result<EntryState, PinError> {
        self.revalidate()?;
        // `FILE_OPEN_FOR_BACKUP_INTENT` is what lets `NtCreateFile` open a
        // directory entry (the NT form of `FILE_FLAG_BACKUP_SEMANTICS`, which
        // every pre-anchored open here relied on); a plain file opens the
        // same as without it, and `FILE_FLAG_OPEN_REPARSE_POINT` opens a
        // reparse point as itself so it is reported, not traversed.
        let handle = match open_anchored(
            self.handle.as_raw_handle() as HANDLE,
            name,
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            FILE_OPEN,
            FILE_OPEN_FOR_BACKUP_INTENT | FILE_FLAG_OPEN_REPARSE_POINT,
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
    /// touching any existing inode. Consumes the pin.
    ///
    /// The bytes go to a fresh temp sibling created through this pinned
    /// directory's path, which is flushed and then renamed onto `name`. An
    /// existing hard link to the old entry keeps the old bytes: only the
    /// directory entry changes. On failure the temp sibling is marked
    /// delete-on-close (write/sync failure) or reported with the directory's
    /// state (rename/verification failure) — nothing is deleted silently.
    ///
    /// Invariant (F05), in three brackets:
    ///
    /// 1. **Pinned window** — validation, temp creation, the byte write, the
    ///    sync, and a final revalidation all happen under the exclusive pin:
    ///    no other handle can open this directory for write or delete access,
    ///    so the visible temp name is not a substitution window and the
    ///    directory's identity is confirmed immediately before the pin drops.
    /// 2. **Release window** — the rename runs with the pin released, because
    ///    the kernel's replace takes a write-class open of the target's
    ///    parent, which this pin's own `FILE_SHARE_READ`-only share refuses
    ///    (the round-6/7 Windows lane: `ERROR_SHARING_VIOLATION` for the
    ///    by-handle `FileRenameInfo` form, the raw-NT class-65 form, and the
    ///    by-path `std::fs::rename` form alike). The window is bounded to the
    ///    single rename call and covered: (i) `ReplaceIfExists` overwrites
    ///    whatever an attacker plants at `name`, so planting gains nothing;
    ///    (ii) an attacker-set reparse point at `name` is either replaced by
    ///    the rename or fails it loudly; and (iii) the directory identity is
    ///    re-verified by the immediate re-open below. The pin's mutual
    ///    exclusion between two concurrent materialization writes narrows to
    ///    this window too: a same-name collision there is never silently
    ///    mixed — the loser's identity verification fails loudly, and the
    ///    published bytes are always one writer's complete content.
    /// 3. **Verification** — the pin is re-opened (the full open-time
    ///    reparse/identity/path validation) and the published `name` is then
    ///    opened anchored to that re-validated directory: it must be a plain
    ///    entry carrying the temp's captured `FILE_ID_INFO` identity. A
    ///    mismatch, a reparse point, or an open failure is a loud typed error
    ///    that describes the target state — a successful substitution can
    ///    never read as a completed write.
    pub fn write_replacing(self, name: &OsStr, bytes: &[u8]) -> io::Result<()> {
        use std::io::Write;
        use std::os::windows::fs::OpenOptionsExt;
        use std::sync::atomic::Ordering;
        self.revalidate().map_err(pin_error_into_io)?;
        // The temp is created through the directory's own path, not an
        // anchored NtCreateFile: Server 2025 rejects the anchored FILE_CREATE
        // with STATUS_INVALID_PARAMETER even though anchored opens of existing
        // entries succeed (recorded in the relative-target spike verdict).
        // This is safe under the exclusive pin — the pin already excludes
        // every concurrent directory writer for the whole window, so the
        // short-lived visible temp name is not a substitution window.
        let mut attempts = 0;
        let (mut file, temp_path) = loop {
            let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let mut temp_name = std::ffi::OsString::from(".");
            temp_name.push(name);
            temp_name.push(format!(".sf-tmp-{}-{sequence}", std::process::id()));
            let temp_path = self.path.join(&temp_name);
            match std::fs::OpenOptions::new()
                // The failure path marks the temp delete-on-close, which
                // requires DELETE access on the handle; std's `write(true)`
                // alone omits it.
                .access_mode(GENERIC_WRITE | DELETE)
                .write(true)
                .create_new(true)
                .share_mode(FILE_SHARE_READ)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&temp_path)
            {
                Ok(file) => break (file, temp_path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists && attempts < 16 => {
                    attempts += 1;
                }
                Err(error) => return Err(error),
            }
        };
        let mut temp_id: Option<FILE_ID_INFO> = None;
        let outcome = file
            .write_all(bytes)
            .and_then(|()| file.sync_all())
            // Capture the temp's identity from its own open handle, while the
            // pin still brackets the directory: this is the id the published
            // entry must carry after the release-window rename.
            .and_then(|()| {
                temp_id = Some(handle_id(file.as_raw_handle() as HANDLE)?);
                Ok(())
            })
            // The destination directory is re-validated immediately before the
            // pin drops too: with the exclusive pin this brackets the whole
            // write window.
            .and_then(|()| self.revalidate().map_err(pin_error_into_io));
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
            return outcome;
        }
        let temp_id = temp_id.expect("temp identity captured after the sync");
        // The rename is path-based and needs the temp handle closed: a rename
        // of an open file conflicts with its own handle.
        drop(file);
        let parent_path = self.path.clone();
        self.release()?;

        // The publish: a plain path rename with the pin released (see the
        // invariant's release window above for why the pin cannot hold here).
        let destination = parent_path.join(name);
        let renamed = std::fs::rename(&temp_path, &destination);
        // Immediately re-open the pin: the full open-time validation (final
        // path, reparse tag, `FILE_ID_INFO` identity) re-establishes that the
        // directory this verification anchors to is the object that was
        // pinned, however the rename landed.
        let repinned = match PinnedDir::open(&parent_path) {
            Ok(repinned) => repinned,
            Err(error) => {
                return Err(io::Error::new(
                    pin_error_into_io(error).kind(),
                    format!(
                        "published write to {} could not be verified: the pinned directory \
                         could not be re-opened after the rename window (rename reported: \
                         {renamed:?})",
                        destination.display(),
                    ),
                ));
            }
        };
        renamed?;
        // Verify the published entry: open it anchored to the re-validated
        // directory, refuse a reparse point, and require the temp's identity.
        let published = open_anchored(
            repinned.handle.as_raw_handle() as HANDLE,
            name,
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            FILE_OPEN,
            FILE_OPEN_FOR_BACKUP_INTENT | FILE_FLAG_OPEN_REPARSE_POINT,
        )
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "published write to {} could not be verified: the renamed entry could \
                     not be re-opened for identity verification: {error} (the rename \
                     reported success; the temp sibling is consumed)",
                    destination.display(),
                ),
            )
        })?;
        let tag = handle_attribute_tag(published.as_raw_handle() as HANDLE).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "published write to {} could not be verified: {error}",
                    destination.display(),
                ),
            )
        })?;
        if tag.ReparseTag != 0 || tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "published write to {} failed verification: the renamed entry is a \
                     reparse point, not the written file; the directory content must be \
                     treated as unverified",
                    destination.display(),
                ),
            ));
        }
        let published_id = handle_id(published.as_raw_handle() as HANDLE)?;
        if published_id.VolumeSerialNumber != temp_id.VolumeSerialNumber
            || published_id.FileId.Identifier != temp_id.FileId.Identifier
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "published write to {} failed verification: the entry at the destination \
                     carries a different file identity than the written temp (substitution \
                     during the release window); the directory content must be treated as \
                     unverified",
                    destination.display(),
                ),
            ));
        }
        Ok(())
    }
}

/// Converts a [`PinError`] into the `io::Error` the `io::Result`-shaped write
/// API reports, keeping the violated invariant in the message text.
fn pin_error_into_io(error: PinError) -> io::Error {
    match error {
        PinError::ReparsePoint(path) => io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("pinned directory is a reparse point: {}", path.display()),
        ),
        PinError::NotDirectory(path) => io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("pinned entry is not a directory: {}", path.display()),
        ),
        PinError::Missing(path) => io::Error::new(
            io::ErrorKind::NotFound,
            format!("pinned entry does not exist: {}", path.display()),
        ),
        PinError::Escaped(path) => io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "entry resolves outside the pinned parent: {}",
                path.display()
            ),
        ),
        PinError::WriteShareHeld(path) => io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "pinned directory is held by another writer; exclusive pin refused: {}",
                path.display()
            ),
        ),
        PinError::IdentityChanged(path) => io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "pinned directory changed identity after validation: {}",
                path.display()
            ),
        ),
        PinError::FinalPathMismatch(path) => io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "opened handle resolves to a different path than requested: {}",
                path.display()
            ),
        ),
        PinError::Io(error) => error,
    }
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

/// Spawns `cmd` suspended, assigns it to a new kill-on-close job, verifies
/// the assignment by reading `IsProcessInJob` back from the kernel, then
/// resumes its threads. The primary thread never runs before the assignment,
/// so no instruction of the child — and therefore no grandchild — can be
/// created before membership is verified. That suspend-before-assign sequence
/// provides the creation-time containment of `PROC_THREAD_ATTRIBUTE_JOB_LIST`
/// without re-implementing std's `Command` spawn over `CreateProcessW` (the
/// attribute-list form itself is proven by the
/// `job_list_attribute_places_the_child_in_the_job_at_creation` test).
///
/// Breakaway is prevented by omission: the job sets only
/// `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, never `JOB_OBJECT_LIMIT_BREAKAWAY_OK`
/// or `JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK`, so a `CREATE_BREAKAWAY_FROM_JOB`
/// process creation for a job member fails at `CreateProcessW` with
/// `ERROR_ACCESS_DENIED` (asserted by a test). Cleanup after a timeout is
/// `terminate_and_wait`, which polls the job until it is empty or a deadline
/// passes and reports survivors.
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
    let assigned = job.assign(&child).and_then(|()| job.verify_member(&child));
    if let Err(error) = assigned {
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
    ///
    /// Only `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` is set. The breakaway flags
    /// (`JOB_OBJECT_LIMIT_BREAKAWAY_OK`, `JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK`)
    /// are deliberately omitted: without them the kernel refuses every
    /// `CREATE_BREAKAWAY_FROM_JOB` process creation for a member of this job,
    /// so no descendant can leave the containment.
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

    /// Whether the kernel reports `process` as a member of this job.
    pub fn contains_process(&self, process: &impl AsRawHandle) -> io::Result<bool> {
        let mut member = FALSE;
        // SAFETY: both handles are live and `member` is a valid 4-byte out-buffer.
        let ok = unsafe {
            IsProcessInJob(
                process.as_raw_handle() as HANDLE,
                self.handle.as_raw_handle() as HANDLE,
                &mut member,
            )
        };
        if ok == FALSE {
            return Err(last_error());
        }
        Ok(member != FALSE)
    }

    /// Fails unless the kernel reports `process` as a member of this job: the
    /// containment claim is verified by read-back, never assumed.
    pub fn verify_member(&self, process: &impl AsRawHandle) -> io::Result<()> {
        if !self.contains_process(process)? {
            return Err(io::Error::other(
                "process is not a member of its containment job (IsProcessInJob read false)",
            ));
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

    use std::ffi::c_void;
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use std::process::{Command, Stdio};
    use windows_sys::Wdk::Storage::FileSystem::{FileRenameInformationEx, NtSetInformationFile};
    use windows_sys::Wdk::Storage::FileSystem::{FILE_CREATE, FILE_NON_DIRECTORY_FILE};
    use windows_sys::Win32::Foundation::ERROR_ACCESS_DENIED;
    use windows_sys::Win32::Security::Authorization::{
        ConvertSecurityDescriptorToStringSecurityDescriptorW, GetNamedSecurityInfoW, SE_FILE_OBJECT,
    };
    use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
    use windows_sys::Win32::Storage::FileSystem::FileRenameInfo;
    use windows_sys::Win32::Storage::FileSystem::{FILE_RENAME_INFO, FILE_RENAME_INFO_0};
    use windows_sys::Win32::System::Threading::{
        CreateProcessW, DeleteProcThreadAttributeList, InitializeProcThreadAttributeList,
        UpdateProcThreadAttribute, CREATE_BREAKAWAY_FROM_JOB, EXTENDED_STARTUPINFO_PRESENT,
        LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_JOB_LIST,
        PROC_THREAD_ATTRIBUTE_PARENT_PROCESS, STARTUPINFOEXW, STARTUPINFOW,
    };

    /// `FILE_RENAME_FLAG_REPLACE_IF_EXISTS` (`FILE_RENAME_INFO.Flags`): the
    /// rename may replace an existing destination entry. windows-sys does not
    /// declare the flag, and the value is fixed by the `FILE_RENAME_INFO`
    /// contract used with `FileRenameInformationEx`. Spike-only: the
    /// production rename is the Win32 class-10 form with `ReplaceIfExists`
    /// (see `rename_handle_into`).
    const FILE_RENAME_FLAG_REPLACE_IF_EXISTS: u32 = 0x0000_0001;

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

    #[test]
    fn pinned_dir_open_fails_loud_when_a_writer_holds_the_directory() {
        let dir = std::env::temp_dir().join(format!("sf-os-pinw-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dir");
        // A handle holding write access on the directory, even one that
        // shares read and write, conflicts with the exclusive pin: the pin
        // must refuse loudly instead of coexisting with an untracked writer.
        let writer = open_entry(
            &dir,
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            OPEN_EXISTING,
            DIRECTORY_FLAGS,
        )
        .expect("writer handle");
        let opened = PinnedDir::open(&dir)
            .err()
            .expect("the pin must fail while a writer holds the directory");
        match opened {
            PinError::WriteShareHeld(path) => {
                assert_eq!(path, dir, "the error names the pinned path");
            }
            other => panic!("expected WriteShareHeld, got {other:?}"),
        }
        drop(writer);
        PinnedDir::open(&dir).expect("pin succeeds once the writer released the directory");
        // A concurrent reader does not block the pin: read sharing stays open.
        let reader = open_entry(
            &dir,
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            OPEN_EXISTING,
            DIRECTORY_FLAGS,
        )
        .expect("reader handle");
        let pinned = PinnedDir::open(&dir).expect("pin succeeds alongside a concurrent reader");
        pinned.revalidate().expect("revalidation of the fresh pin");
        drop(reader);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn revalidation_refuses_a_swapped_directory_identity() {
        let a_path = std::env::temp_dir().join(format!("sf-os-swap-a-{}", std::process::id()));
        let b_path = std::env::temp_dir().join(format!("sf-os-swap-b-{}", std::process::id()));
        let _ = (
            std::fs::remove_dir_all(&a_path),
            std::fs::remove_dir_all(&b_path),
        );
        std::fs::create_dir_all(&a_path).expect("dir a");
        std::fs::create_dir_all(&b_path).expect("dir b");
        let real = PinnedDir::open(&a_path).expect("pin a");
        let other = PinnedDir::open(&b_path).expect("pin b");
        // Revalidation is directly invocable, so the swap is staged in the
        // structure: a handle on another directory behind the recorded
        // identity and path.
        let swapped = PinnedDir {
            handle: other.handle,
            path: real.path.clone(),
            id: real.id,
        };
        match swapped.revalidate() {
            Err(PinError::IdentityChanged(path)) => {
                // The reported path is the pinned handle's resolved final
                // path: it carries the `\\?\` device prefix and long-form
                // components, so it names `a_path` up to the device prefix,
                // case, separators and 8.3 expansion of `temp_dir` — never a
                // different directory.
                let expected = long_path(&a_path).unwrap_or(a_path.clone());
                assert!(
                    same_path(&strip_device_prefix(&path), &expected.to_string_lossy()),
                    "the error names the validated path: {path:?} vs {a_path:?}"
                );
            }
            other => panic!("expected IdentityChanged, got {other:?}"),
        }
        drop(real);
        let _ = std::fs::remove_dir_all(&a_path);
        let _ = std::fs::remove_dir_all(&b_path);
    }

    /// The Win32 relative-target rename (`SetFileInformationByHandle` with
    /// class `FileRenameInformation` — information class 10 — and a
    /// `RootDirectory` handle). Spike-only: production renames with the
    /// class-10 absolute `\\?\` target and no `RootDirectory` (see
    /// `rename_handle_into`); the relative form is one of the three this
    /// platform rejects.
    fn set_rename_info_with_root(file: HANDLE, root_dir: HANDLE, name: &OsStr) -> io::Result<()> {
        let name_wide: Vec<u16> = name.encode_wide().collect();
        let name_bytes = name_wide.len() * mem::size_of::<u16>();
        let size = mem::size_of::<FILE_RENAME_INFO>() + name_bytes;
        let mut buffer = vec![0u64; size.div_ceil(mem::size_of::<u64>())];
        let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFO>();
        // SAFETY: `buffer` is zeroed, 8-byte aligned and at least `size` bytes, so the header and `name_bytes` of FileName are in bounds and initialized.
        unsafe {
            (*info).Anonymous = FILE_RENAME_INFO_0 {
                ReplaceIfExists: true,
            };
            (*info).RootDirectory = root_dir;
            (*info).FileNameLength = name_bytes as u32;
            let destination = std::ptr::addr_of_mut!((*info).FileName).cast::<u16>();
            std::ptr::copy_nonoverlapping(name_wide.as_ptr(), destination, name_wide.len());
        }
        // SAFETY: `file` is a live handle with DELETE access; `info` points at `size` initialized bytes that outlive the call.
        let ok =
            unsafe { SetFileInformationByHandle(file, FileRenameInfo, info.cast(), size as u32) };
        if ok == FALSE {
            return Err(last_error());
        }
        Ok(())
    }

    /// The NT relative-target rename (`NtSetInformationFile` with class
    /// `FileRenameInformationEx` — information class 65 — and a
    /// `RootDirectory` handle). Spike-only: one of the three forms this
    /// platform rejects (see the spike test's verdict comment).
    fn set_rename_info_ex_with_root(
        file: HANDLE,
        root_dir: HANDLE,
        name: &OsStr,
    ) -> io::Result<()> {
        let name_wide: Vec<u16> = name.encode_wide().collect();
        let name_bytes = name_wide.len() * mem::size_of::<u16>();
        let size = mem::size_of::<FILE_RENAME_INFO>() + name_bytes;
        let mut buffer = vec![0u64; size.div_ceil(mem::size_of::<u64>())];
        let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFO>();
        // SAFETY: `buffer` is zeroed, 8-byte aligned and at least `size` bytes, so the header and `name_bytes` of FileName are in bounds and initialized.
        unsafe {
            (*info).Anonymous = FILE_RENAME_INFO_0 {
                Flags: FILE_RENAME_FLAG_REPLACE_IF_EXISTS,
            };
            (*info).RootDirectory = root_dir;
            (*info).FileNameLength = name_bytes as u32;
            let destination = std::ptr::addr_of_mut!((*info).FileName).cast::<u16>();
            std::ptr::copy_nonoverlapping(name_wide.as_ptr(), destination, name_wide.len());
        }
        // SAFETY: IO_STATUS_BLOCK is plain data; all-zero is a valid initial value.
        let mut status_block: IO_STATUS_BLOCK = unsafe { mem::zeroed() };
        // SAFETY: `file` is a live handle with DELETE access; `info` points at `size` initialized bytes that outlive the call; `status_block` is a live out-struct.
        let status = unsafe {
            NtSetInformationFile(
                file,
                &mut status_block,
                info.cast(),
                size as u32,
                FileRenameInformationEx,
            )
        };
        if status < 0 {
            return Err(nt_error(status));
        }
        Ok(())
    }

    /// The rename-form platform verdicts, for the windows portability lane:
    /// `cargo test -p sea-forge-os --lib -- --include-ignored`.
    ///
    /// Recorded platform verdicts (Server 2025 x64, capability-harvest CI
    /// rounds 2-7). THREE rename forms are rejected with
    /// ERROR_INVALID_PARAMETER — (1) the Win32 class-10 relative target,
    /// `SetFileInformationByHandle(FileRenameInformation)` + `RootDirectory`
    /// (the form the prior branch shipped until 53c3bd9a), (2) the NT
    /// class-65 relative target,
    /// `NtSetInformationFile(FileRenameInformationEx)` + `RootDirectory`, and
    /// (3) the NT class-65 absolute `\??\` target with `RootDirectory` NULL
    /// (the round-3 production attempt). The anchored `NtCreateFile`
    /// `FILE_CREATE` temp is rejected with `STATUS_INVALID_PARAMETER` on the
    /// same platform (round 4), so the temp is created through the
    /// directory's path.
    ///
    /// Round 6 added the decisive verdict for the publish step: with the
    /// exclusive pin held, EVERY rename-onto-a-child is refused with
    /// `ERROR_SHARING_VIOLATION` (32) — by-handle `FileRenameInfo` (absolute
    /// `\\?\` target, round 6) AND by-path `std::fs::rename` (round 7) —
    /// because the kernel's replace takes a write-class open of the target's
    /// parent and the pin's `FILE_SHARE_READ`-only share refuses any new
    /// write-access open of the directory. `write_replacing` therefore
    /// releases the pin around the rename and re-verifies by file identity
    /// (see its invariant note). This spike re-probes every recorded form on
    /// each run and RECORDS the verdicts instead of asserting, so a platform
    /// that starts accepting a rejected form is visible in the log rather
    /// than silently unexercised.
    #[test]
    #[ignore = "windows portability lane: rename-form verdicts (Server 2025 x64 rejected the \
                class-10 Win32 relative, class-65 NT relative, and class-65 NT absolute rename \
                forms with ERROR_INVALID_PARAMETER and the anchored FILE_CREATE temp with \
                STATUS_INVALID_PARAMETER; with the exclusive pin held, renames onto a child are \
                refused ERROR_SHARING_VIOLATION in both the by-handle FileRenameInfo and the \
                by-path std::fs::rename forms); run with --include-ignored"]
    fn relative_target_rename_platform_verdict_spike() {
        let dir = std::env::temp_dir().join(format!("sf-os-spike-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dir");
        let pinned = PinnedDir::open(&dir).expect("pin");
        let root = pinned.handle.as_raw_handle() as HANDLE;
        use std::io::Write as _;

        // (a) anchored creation while pinned exclusively: verdict recorded
        // (Server 2025 rejects it), never asserted; the probes fall back to a
        // path-created temp so the later verdicts stay reachable.
        let anchored = open_anchored(
            root,
            OsStr::new("a.txt"),
            GENERIC_WRITE | DELETE,
            FILE_SHARE_READ,
            FILE_CREATE,
            FILE_NON_DIRECTORY_FILE | FILE_FLAG_OPEN_REPARSE_POINT,
        );
        let mut file = match anchored {
            Ok(handle) => {
                println!("VERDICT: anchored NtCreateFile FILE_CREATE: ACCEPTED");
                std::fs::File::from(handle)
            }
            Err(error) => {
                println!("VERDICT: anchored NtCreateFile FILE_CREATE: REJECTED ({error})");
                std::fs::OpenOptions::new()
                    .access_mode(GENERIC_WRITE | DELETE)
                    .write(true)
                    .create_new(true)
                    .share_mode(FILE_SHARE_READ)
                    .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                    .open(dir.join("a.txt"))
                    .expect("path-created temp a.txt")
            }
        };
        file.write_all(b"one").expect("write temp");
        file.sync_all().expect("sync temp");

        // (b) class 65 relative form: verdict recorded, never asserted.
        match set_rename_info_ex_with_root(
            file.as_raw_handle() as HANDLE,
            root,
            OsStr::new("b.txt"),
        ) {
            Ok(()) => {
                assert_eq!(
                    std::fs::read(dir.join("b.txt")).expect("read renamed"),
                    b"one",
                    "the relative rename landed outside the pinned directory"
                );
                println!("VERDICT: FileRenameInformationEx + RootDirectory: ACCEPTED");
            }
            Err(error) => {
                println!("VERDICT: FileRenameInformationEx + RootDirectory: REJECTED ({error})");
            }
        }
        drop(file);

        // (c) class 10 relative form: verdict recorded, never asserted.
        let file = open_anchored(
            root,
            OsStr::new("c.txt"),
            GENERIC_WRITE | DELETE,
            FILE_SHARE_READ,
            FILE_CREATE,
            FILE_NON_DIRECTORY_FILE | FILE_FLAG_OPEN_REPARSE_POINT,
        );
        let file = match file {
            Ok(handle) => std::fs::File::from(handle),
            Err(error) => {
                println!("VERDICT: anchored NtCreateFile FILE_CREATE (second): REJECTED ({error})");
                std::fs::OpenOptions::new()
                    .access_mode(GENERIC_WRITE | DELETE)
                    .write(true)
                    .create_new(true)
                    .share_mode(FILE_SHARE_READ)
                    .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                    .open(dir.join("c.txt"))
                    .expect("path-created temp c.txt")
            }
        };
        let mut file = file;
        file.write_all(b"two").expect("write second temp");
        file.sync_all().expect("sync second temp");
        match set_rename_info_with_root(file.as_raw_handle() as HANDLE, root, OsStr::new("d.txt")) {
            Ok(()) => {
                assert_eq!(
                    std::fs::read(dir.join("d.txt")).expect("read renamed"),
                    b"two",
                    "the relative rename landed outside the pinned directory"
                );
                println!(
                    "VERDICT: SetFileInformationByHandle(FileRenameInfo) + RootDirectory: ACCEPTED"
                );
            }
            Err(error) => {
                println!(
                    "VERDICT: SetFileInformationByHandle(FileRenameInfo) + RootDirectory: \
                     REJECTED ({error})"
                );
            }
        }
        drop(file);

        // (e) by-handle rename onto a child with the pin still held: the
        // shipped class-10 ABSOLUTE target form. Verdict recorded — the
        // round-6 expectation is ERROR_SHARING_VIOLATION (the replace takes a
        // write-class open of the pinned parent).
        let file = std::fs::OpenOptions::new()
            .access_mode(GENERIC_WRITE | DELETE)
            .write(true)
            .create_new(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(dir.join("e.txt"))
            .expect("temp e.txt");
        let mut file = file;
        file.write_all(b"five").expect("write e temp");
        file.sync_all().expect("sync e temp");
        let absolute_target = {
            let mut target = dir.as_os_str().to_os_string();
            target.push("\\e-dest.txt");
            PathBuf::from(format!(r"\\?\{}", target.to_string_lossy()))
        };
        match set_rename_info_absolute(file.as_raw_handle() as HANDLE, &absolute_target) {
            Ok(()) => {
                assert_eq!(
                    std::fs::read(dir.join("e-dest.txt")).expect("read renamed"),
                    b"five",
                    "the absolute rename landed outside the pinned directory"
                );
                println!(
                    "VERDICT: SetFileInformationByHandle(FileRenameInfo) absolute under pin: \
                     ACCEPTED"
                );
            }
            Err(error) => {
                println!(
                    "VERDICT: SetFileInformationByHandle(FileRenameInfo) absolute under pin: \
                     REJECTED ({error})"
                );
            }
        }
        drop(file);

        // (f) by-path rename onto a child with the pin still held
        // (`std::fs::rename`, the round-7 production publish form). Verdict
        // recorded — the round-7 lane expectation is ERROR_SHARING_VIOLATION.
        std::fs::write(dir.join("f.txt"), b"six").expect("temp f.txt");
        match std::fs::rename(dir.join("f.txt"), dir.join("f-dest.txt")) {
            Ok(()) => {
                assert_eq!(
                    std::fs::read(dir.join("f-dest.txt")).expect("read renamed"),
                    b"six",
                    "the by-path rename landed outside the pinned directory"
                );
                println!("VERDICT: std::fs::rename by path under pin: ACCEPTED");
            }
            Err(error) => {
                println!("VERDICT: std::fs::rename by path under pin: REJECTED ({error})");
            }
        }

        // (d) the exclusive pin holds while every probed operation ran: a
        // direct write-class open of the pinned directory is refused.
        match open_entry(
            &dir,
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            OPEN_EXISTING,
            DIRECTORY_FLAGS,
        ) {
            Err(error) => assert_eq!(
                error.raw_os_error(),
                Some(ERROR_SHARING_VIOLATION as i32),
                "unexpected error: {error}"
            ),
            Ok(_) => panic!("a write handle was opened on the exclusively pinned directory"),
        }
        drop(pinned);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The by-handle absolute-target rename (`SetFileInformationByHandle`
    /// with class `FileRenameInfo`, `ReplaceIfExists=TRUE`, `RootDirectory`
    /// NULL, and a fully-qualified `\\?\` target). Spike-only: the form the
    /// pre-anchored implementation shipped for the project's lifetime; under
    /// the exclusive pin it is expected to be refused with
    /// `ERROR_SHARING_VIOLATION` (see the spike's recorded verdicts).
    fn set_rename_info_absolute(file: HANDLE, target: &Path) -> io::Result<()> {
        let target_wide: Vec<u16> = target.as_os_str().encode_wide().collect();
        let target_bytes = target_wide.len() * mem::size_of::<u16>();
        let size = mem::size_of::<FILE_RENAME_INFO>() + target_bytes;
        let mut buffer = vec![0u64; size.div_ceil(mem::size_of::<u64>())];
        let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFO>();
        // SAFETY: `buffer` is zeroed, 8-byte aligned and at least `size` bytes, so the header and `target_bytes` of FileName are in bounds and initialized.
        unsafe {
            (*info).Anonymous = FILE_RENAME_INFO_0 {
                ReplaceIfExists: true,
            };
            (*info).RootDirectory = std::ptr::null_mut();
            (*info).FileNameLength = target_bytes as u32;
            let destination = std::ptr::addr_of_mut!((*info).FileName).cast::<u16>();
            std::ptr::copy_nonoverlapping(target_wide.as_ptr(), destination, target_wide.len());
        }
        // SAFETY: `file` is a live handle with DELETE access; `info` points at `size` initialized bytes that outlive the call.
        let ok =
            unsafe { SetFileInformationByHandle(file, FileRenameInfo, info.cast(), size as u32) };
        if ok == FALSE {
            return Err(last_error());
        }
        Ok(())
    }

    /// A one-attribute `PROC_THREAD_ATTRIBUTE_LIST` for direct
    /// `CreateProcessW` calls in tests.
    struct ProcThreadAttributeList {
        /// Opaque list storage; `u64` elements keep it 8-byte aligned because
        /// the kernel stores pointers inside the list.
        buffer: Vec<u64>,
    }

    impl ProcThreadAttributeList {
        fn new(count: usize) -> io::Result<Self> {
            let mut size = 0usize;
            // SAFETY: a null list with count 0 is the documented size query; `size` is a valid out-pointer.
            unsafe { InitializeProcThreadAttributeList(null_mut(), count as u32, 0, &mut size) };
            if size == 0 {
                return Err(last_error());
            }
            let mut buffer = vec![0u64; size.div_ceil(mem::size_of::<u64>())];
            // SAFETY: `buffer` holds at least `size` bytes of aligned storage for the list.
            let ok = unsafe {
                InitializeProcThreadAttributeList(
                    buffer.as_mut_ptr().cast(),
                    count as u32,
                    0,
                    &mut size,
                )
            };
            if ok == FALSE {
                return Err(last_error());
            }
            Ok(Self { buffer })
        }

        fn set(&mut self, attribute: usize, value: *const c_void) -> io::Result<()> {
            // SAFETY: the list was initialized above; `value` names `size_of::<HANDLE>()` bytes that outlive the CreateProcessW call made with this list.
            let ok = unsafe {
                UpdateProcThreadAttribute(
                    self.buffer.as_mut_ptr().cast(),
                    0,
                    attribute,
                    value,
                    mem::size_of::<HANDLE>(),
                    null_mut(),
                    null(),
                )
            };
            if ok == FALSE {
                return Err(last_error());
            }
            Ok(())
        }

        fn ptr(&self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
            self.buffer.as_ptr().cast_mut().cast()
        }
    }

    impl Drop for ProcThreadAttributeList {
        fn drop(&mut self) {
            // SAFETY: the list was initialized once and is deleted exactly once.
            unsafe { DeleteProcThreadAttributeList(self.buffer.as_mut_ptr().cast()) };
        }
    }

    /// Creates `command_line` through `CreateProcessW` with `creation_flags`
    /// and, when given, an attribute list (which implies
    /// `EXTENDED_STARTUPINFO_PRESENT` and rides in a `STARTUPINFOEXW`).
    /// Returns the process and thread handles and the pid.
    fn create_process_raw(
        command_line: &str,
        creation_flags: u32,
        attributes: Option<&ProcThreadAttributeList>,
    ) -> io::Result<(OwnedHandle, OwnedHandle, u32)> {
        let mut command_line: Vec<u16> = std::ffi::OsStr::new(command_line)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: STARTUPINFOEXW and PROCESS_INFORMATION are plain data; all-zero is a valid initial value with the size fields set below.
        let mut startup: STARTUPINFOEXW = unsafe { mem::zeroed() };
        let mut info: PROCESS_INFORMATION = unsafe { mem::zeroed() };
        let mut flags = creation_flags;
        match attributes {
            Some(list) => {
                startup.StartupInfo.cb = mem::size_of::<STARTUPINFOEXW>() as u32;
                startup.lpAttributeList = list.ptr();
                flags |= EXTENDED_STARTUPINFO_PRESENT;
            }
            None => {
                startup.StartupInfo.cb = mem::size_of::<STARTUPINFOW>() as u32;
            }
        }
        // SAFETY: `command_line` is a NUL-terminated mutable wide string that outlives the call; the attribute list, when given, is initialized and its referenced values outlive the call; `startup` and `info` are live structs.
        let ok = unsafe {
            CreateProcessW(
                null(),
                command_line.as_mut_ptr(),
                null(),
                null(),
                FALSE,
                flags,
                null(),
                null(),
                (&startup as *const STARTUPINFOEXW).cast(),
                &mut info,
            )
        };
        if ok == FALSE {
            return Err(last_error());
        }
        // SAFETY: CreateProcessW succeeded, so both handles are fresh and owned by us.
        let process = unsafe { OwnedHandle::from_raw_handle(info.hProcess) };
        let thread = unsafe { OwnedHandle::from_raw_handle(info.hThread) };
        Ok((process, thread, info.dwProcessId))
    }

    #[test]
    fn job_list_attribute_places_the_child_in_the_job_at_creation() {
        // The direct equivalent of heel's creation-time assignment: a child
        // created with PROC_THREAD_ATTRIBUTE_JOB_LIST is a job member before
        // its first instruction. This documents the mechanism the
        // suspend-before-assign sequence in `spawn_in_job` is equivalent to.
        let job = Job::kill_on_close().expect("job");
        let mut list = ProcThreadAttributeList::new(1).expect("attribute list");
        let job_raw = job.handle.as_raw_handle() as HANDLE;
        list.set(
            PROC_THREAD_ATTRIBUTE_JOB_LIST as usize,
            (&job_raw as *const HANDLE).cast(),
        )
        .expect("set the job-list attribute");
        let (process, _thread, _pid) =
            create_process_raw("cmd.exe /c exit 0", 0, Some(&list)).expect("create process");
        assert!(
            job.contains_process(&process).expect("membership query"),
            "a child created with PROC_THREAD_ATTRIBUTE_JOB_LIST is not in the job"
        );
    }

    #[test]
    fn breakaway_from_job_is_refused_when_the_job_forbids_it() {
        // NT rule: CREATE_BREAKAWAY_FROM_JOB fails at CreateProcessW with
        // ERROR_ACCESS_DENIED unless the job sets
        // JOB_OBJECT_LIMIT_BREAKAWAY_OK. The job from `spawn_in_job`
        // deliberately omits the flag, so a workload's breakaway attempt
        // cannot leave the containment.
        let mut command = Command::new("cmd");
        command
            .args(["/c", "ping -n 20 127.0.0.1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let (mut child, job) = spawn_in_job(&mut command).expect("child in job");
        let mut list = ProcThreadAttributeList::new(1).expect("attribute list");
        // `UpdateProcThreadAttribute` copies `cbSize` bytes FROM `lpValue`:
        // `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS` takes a pointer to the
        // handle value, so the handle must go through a local and only the
        // local's address is passed. Passing the handle itself as the pointer
        // dereferences a low, unmapped address (STATUS_ACCESS_VIOLATION).
        let parent_handle: HANDLE = child.as_raw_handle() as HANDLE;
        list.set(
            PROC_THREAD_ATTRIBUTE_PARENT_PROCESS as usize,
            (&parent_handle as *const HANDLE).cast(),
        )
        .expect("set the parent-process attribute");
        match create_process_raw("cmd.exe /c exit 0", CREATE_BREAKAWAY_FROM_JOB, Some(&list)) {
            Ok(_) => panic!(
                "a breakaway creation succeeded although the job forbids breakaway; \
                 the workload escaped containment"
            ),
            Err(error) => assert_eq!(
                error.raw_os_error(),
                Some(ERROR_ACCESS_DENIED as i32),
                "breakaway must be refused with ERROR_ACCESS_DENIED, got {error}"
            ),
        }
        job.terminate_and_wait(Duration::from_secs(5))
            .expect("job emptied after cleanup");
        let _ = child.wait();
    }
}
