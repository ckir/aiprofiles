//! Windows launcher: a direct child process with console control-event handling and a job object so an
//! interrupted wrapper leaves no orphan (spec §23.2, §24; design §7.6).

use std::io::{self, Write};
use std::mem::{size_of, zeroed};
use std::os::windows::io::AsRawHandle;
use std::ptr::null;
use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{
    DUPLICATE_HANDLE_OPTIONS, DuplicateHandle, FALSE, HANDLE, TRUE,
};
use windows_sys::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows_sys::Win32::System::Console::{
    CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT,
    SetConsoleCtrlHandler,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, INFINITE, WaitForSingleObject};
use windows_sys::core::BOOL;

use super::{LaunchOutcome, LaunchPlan};
use crate::error::{Error, Result};

/// Handles the console control handler needs, as raw values (raw handles are not `Send`/`Sync`).
struct Published {
    job: usize,
    child: usize,
}

static PUBLISHED: OnceLock<Published> = OnceLock::new();

/// Debug-only hook: sleep this many milliseconds immediately before spawning (design §8.4).
#[cfg(debug_assertions)]
const PAUSE_ENV: &str = "AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS";

pub(super) fn launch(plan: &LaunchPlan, verbose: bool) -> Result<LaunchOutcome> {
    let launch_error =
        |source: io::Error| Error::Launch { executable: plan.executable.clone(), source };

    // 1. Job: the wrapper joins a kill-on-close job, so its children die if it is killed.
    // SAFETY: plain Win32 calls with valid arguments; the job handle stays open until process exit.
    let job = unsafe { CreateJobObjectW(null(), null()) };
    if job.is_null() {
        return Err(launch_error(io::Error::last_os_error()));
    }
    if !set_kill_on_close(job, true) {
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
        let _ = PUBLISHED.set(Published { job: job as usize, child: duplicate as usize });
    }

    // 4. Wait.
    let status = child.wait().map_err(launch_error)?;

    // 5. Release the job so processes the agent left running survive the wrapper's exit.
    if !set_kill_on_close(job, false) && verbose {
        let _ = writeln!(
            io::stderr(),
            "agent-profile: could not release the job object: {}",
            io::Error::last_os_error()
        );
    }

    // 6. The child's full exit code; `main` exits with it.
    Ok(LaunchOutcome::Exited(status.code().unwrap_or(1)))
}

fn set_kill_on_close(job: HANDLE, kill: bool) -> bool {
    // SAFETY: an all-zero JOBOBJECT_EXTENDED_LIMIT_INFORMATION is a valid "no limits" value.
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    if kill {
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    }
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
                set_kill_on_close(published.job as HANDLE, false);
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
