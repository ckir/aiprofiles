//! Windows launcher: a direct child process with console control-event handling and a job object so an
//! interrupted wrapper leaves no orphan (spec §23.2, §24; design §7.6).

use std::io::{self, Write};
use std::mem::{size_of, zeroed};
use std::os::windows::io::AsRawHandle;
use std::ptr::{null, null_mut};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use windows_sys::Win32::Foundation::{
    DUPLICATE_HANDLE_OPTIONS, DuplicateHandle, FALSE, HANDLE, TRUE,
};
use windows_sys::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows_sys::Win32::System::Console::{
    CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT,
    SetConsoleCtrlHandler,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob, JOB_OBJECT_LIMIT_BREAKAWAY_OK,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    QueryInformationJobObject, SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, INFINITE, WaitForSingleObject};
use windows_sys::core::BOOL;

use super::{LaunchOutcome, LaunchPlan};
use crate::error::{Error, Result};

/// Handles the console control handler needs, as raw values (raw handles are not `Send`/`Sync`).
struct Published {
    job: usize,
    child: usize,
    /// The job's limit flags once released: the inherited breakaway flag only.
    breakaway: u32,
}

static PUBLISHED: OnceLock<Published> = OnceLock::new();

/// Set by the first launch. The control handler and `PUBLISHED` describe a single child, so a second launch in
/// the same process is refused instead of silently running without its close-event guarantees.
static LAUNCHED: AtomicBool = AtomicBool::new(false);

/// `CREATE_BREAKAWAY_FROM_JOB` fails with access denied when the creating process's innermost job has neither
/// `JOB_OBJECT_LIMIT_BREAKAWAY_OK` nor `JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK` (measured). The wrapper's job
/// therefore allows breakaway exactly when the job the wrapper was started in allows it, so the agent's breakaway
/// attempts succeed or fail as under direct invocation. The silent flag is never copied: it would take the agent
/// itself out of the wrapper's kill-on-close job.
/// Outside any job, breakaway is allowed. If the caller's job cannot be queried, breakaway is not allowed: the
/// agent then keeps every process inside the kill-on-close job.
fn breakaway_flag(in_job: bool, caller_limits: Option<u32>) -> u32 {
    match (in_job, caller_limits) {
        (false, _) => JOB_OBJECT_LIMIT_BREAKAWAY_OK,
        (true, Some(limits))
            if limits & (JOB_OBJECT_LIMIT_BREAKAWAY_OK | JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK)
                != 0 =>
        {
            JOB_OBJECT_LIMIT_BREAKAWAY_OK
        }
        (true, Some(_)) => 0,
        (true, None) => 0,
    }
}

/// The breakaway flag of the job this process is running in, per `breakaway_flag`.
fn inherited_breakaway_flag() -> u32 {
    let mut in_job = FALSE;
    // SAFETY: a null job handle asks whether the process is in any job; `in_job` is a valid out pointer.
    if unsafe { IsProcessInJob(GetCurrentProcess(), null_mut(), &mut in_job) } == FALSE {
        return breakaway_flag(true, None);
    }
    if in_job == FALSE {
        return breakaway_flag(false, None);
    }
    // SAFETY: an all-zero JOBOBJECT_EXTENDED_LIMIT_INFORMATION is a valid out buffer of the right size.
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    // SAFETY: a null job handle queries the job this process is associated with; the buffer and size match.
    let queried = unsafe {
        QueryInformationJobObject(
            null_mut(),
            JobObjectExtendedLimitInformation,
            (&mut info as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            null_mut(),
        )
    };
    if queried == FALSE {
        return breakaway_flag(true, None);
    }
    breakaway_flag(true, Some(info.BasicLimitInformation.LimitFlags))
}

/// Debug-only hook: sleep this many milliseconds immediately before spawning (design §8.4).
#[cfg(debug_assertions)]
const PAUSE_ENV: &str = "AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS";

pub(super) fn launch(plan: &LaunchPlan, verbose: bool) -> Result<LaunchOutcome> {
    let launch_error =
        |source: io::Error| Error::Launch { executable: plan.executable.clone(), source };
    claim_single_launch().map_err(launch_error)?;

    // 1. Job: the wrapper joins a kill-on-close job, so its children die if it is killed. Breakaway is allowed in
    // the new job exactly when the job the wrapper started in allows it.
    let breakaway = inherited_breakaway_flag();
    // SAFETY: plain Win32 calls with valid arguments; the job handle stays open until process exit.
    let job = unsafe { CreateJobObjectW(null(), null()) };
    if job.is_null() {
        return Err(launch_error(io::Error::last_os_error()));
    }
    if !set_limits(job, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | breakaway) {
        return Err(launch_error(io::Error::last_os_error()));
    }
    // SAFETY: `job` is a valid job handle and `GetCurrentProcess` is always valid.
    if unsafe { AssignProcessToJobObject(job, GetCurrentProcess()) } == FALSE {
        return Err(launch_error(io::Error::last_os_error()));
    }

    // 2. Handler: survive Ctrl-C and Ctrl-Break; the agent shares the console and handles them itself.
    // SAFETY: `handler` is a valid `extern "system"` routine for the life of the process.
    if unsafe { SetConsoleCtrlHandler(Some(handler), TRUE) } == FALSE {
        return Err(launch_error(io::Error::last_os_error()));
    }

    #[cfg(debug_assertions)]
    if let Some(ms) = std::env::var_os(PAUSE_ENV).and_then(|v| v.to_str()?.parse::<u64>().ok()) {
        let mut stderr = io::stderr();
        let _ = writeln!(stderr, "agent-profile: debug: paused before spawn");
        let _ = stderr.flush();
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }

    // 3. Spawn directly and publish the handles for the handler.
    let mut child = plan.command().spawn().map_err(launch_error)?;
    let mut duplicate: HANDLE = std::ptr::null_mut();
    // SAFETY: duplicates the live child handle into this process with SYNCHRONIZE access only.
    let duplicated = unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            child.as_raw_handle() as HANDLE,
            GetCurrentProcess(),
            &mut duplicate,
            SYNCHRONIZE,
            FALSE,
            0 as DUPLICATE_HANDLE_OPTIONS,
        )
    };
    if duplicated != FALSE {
        let _ =
            PUBLISHED.set(Published { job: job as usize, child: duplicate as usize, breakaway });
    }

    // 4. Wait.
    let status = child.wait().map_err(launch_error)?;

    // 5. Release the job so processes the agent left running survive the wrapper's exit.
    if !set_limits(job, breakaway) && verbose {
        let _ = writeln!(
            io::stderr(),
            "agent-profile: could not release the job object: {}",
            io::Error::last_os_error()
        );
    }

    // 6. The child's full exit code; `main` exits with it.
    Ok(LaunchOutcome::Exited(status.code().unwrap_or(1)))
}

/// Refuses every launch after the first in this process.
fn claim_single_launch() -> io::Result<()> {
    if LAUNCHED.swap(true, Ordering::SeqCst) {
        return Err(io::Error::other("the Windows launcher can run only once per process"));
    }
    Ok(())
}

fn set_limits(job: HANDLE, limit_flags: u32) -> bool {
    // SAFETY: an all-zero JOBOBJECT_EXTENDED_LIMIT_INFORMATION is a valid "no limits" value.
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    info.BasicLimitInformation.LimitFlags = limit_flags;
    // SAFETY: `info` is a correctly sized, initialised structure for this information class.
    unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) != FALSE
    }
}

unsafe extern "system" fn handler(event: u32) -> BOOL {
    match event {
        CTRL_C_EVENT | CTRL_BREAK_EVENT => TRUE,
        CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT => match PUBLISHED.get() {
            Some(published) => {
                // SAFETY: `child` is an owned SYNCHRONIZE duplicate that is never closed.
                unsafe { WaitForSingleObject(published.child as HANDLE, INFINITE) };
                set_limits(published.job as HANDLE, published.breakaway);
                // Never return: returning lets Windows terminate the wrapper before `main` exits with the
                // agent's code. Windows' own close timeout remains the backstop.
                loop {
                    std::thread::park();
                }
            }
            None => FALSE,
        },
        _ => FALSE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_first_launch_in_a_process_is_allowed() {
        // Unit tests never launch, so this test owns the process-wide flag.
        assert!(claim_single_launch().is_ok());
        let error = claim_single_launch().unwrap_err();
        assert!(error.to_string().contains("only once per process"), "{error}");
    }

    #[test]
    fn breakaway_mirrors_the_callers_job() {
        const OTHER: u32 = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        assert_eq!(breakaway_flag(false, None), JOB_OBJECT_LIMIT_BREAKAWAY_OK);
        assert_eq!(
            breakaway_flag(true, Some(OTHER | JOB_OBJECT_LIMIT_BREAKAWAY_OK)),
            JOB_OBJECT_LIMIT_BREAKAWAY_OK
        );
        // Silent breakaway in the caller's job also lets an explicit breakaway succeed; the wrapper copies it as the
        // non-silent flag so the agent itself stays in the kill-on-close job.
        assert_eq!(
            breakaway_flag(true, Some(OTHER | JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK)),
            JOB_OBJECT_LIMIT_BREAKAWAY_OK
        );
        assert_eq!(breakaway_flag(true, Some(OTHER)), 0);
        assert_eq!(breakaway_flag(true, None), 0);
    }
}
