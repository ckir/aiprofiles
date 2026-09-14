//! Test-only Windows console driver (SP1 design §8.4). Never ships.
//!
//! Usage: `console-driver <result-file> <none|ctrl-c|ctrl-break> <none|report|pause-marker> <program> [args...]`
//!
//! A test starts this driver with `CREATE_NEW_CONSOLE`, so the driver, the program it runs and the
//! program's children share a console that the test process is not attached to. The driver re-enables
//! Ctrl-C processing (an "ignore" attribute is inherited from the test runner's process chain), swallows
//! events itself, runs the program with piped stdout and stderr drained on separate threads, waits for the
//! readiness signal, sends the event to its whole console, waits for the program, and writes
//! `{"exit_code": N, "stdout": "...", "stderr": "..."}` (or `{"error": "...", ...}`) to the result file.

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("console-driver: Windows only");
    std::process::ExitCode::from(125)
}

#[cfg(windows)]
fn main() -> std::process::ExitCode {
    match windows::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("console-driver: {message}");
            std::process::ExitCode::from(125)
        }
    }
}

#[cfg(windows)]
mod windows {
    use std::io::{BufRead, BufReader, Read};
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    use windows_sys::Win32::Foundation::{FALSE, TRUE};
    use windows_sys::Win32::System::Console::{
        CTRL_BREAK_EVENT, CTRL_C_EVENT, GenerateConsoleCtrlEvent, SetConsoleCtrlHandler,
    };
    use windows_sys::core::BOOL;

    const PAUSE_MARKER: &str = "agent-profile: debug: paused before spawn";
    const READY_TIMEOUT: Duration = Duration::from_secs(30);
    const EXIT_TIMEOUT: Duration = Duration::from_secs(60);

    unsafe extern "system" fn swallow(_event: u32) -> BOOL {
        TRUE
    }

    pub fn run() -> Result<(), String> {
        let mut args = std::env::args_os().skip(1);
        let usage = "usage: console-driver <result-file> <event> <readiness> <program> [args...]";
        let result_file = args.next().ok_or(usage)?;
        let event = args.next().and_then(|v| v.into_string().ok()).ok_or(usage)?;
        let readiness = args.next().and_then(|v| v.into_string().ok()).ok_or(usage)?;
        let program = args.next().ok_or(usage)?;
        let event = match event.as_str() {
            "none" => None,
            "ctrl-c" => Some(CTRL_C_EVENT),
            "ctrl-break" => Some(CTRL_BREAK_EVENT),
            other => return Err(format!("unknown event {other:?}")),
        };
        if !matches!(readiness.as_str(), "none" | "report" | "pause-marker") {
            return Err(format!("unknown readiness {readiness:?}"));
        }

        // SAFETY: plain Win32 calls; `swallow` is a valid routine for the life of the process.
        unsafe {
            if SetConsoleCtrlHandler(None, FALSE) == FALSE
                || SetConsoleCtrlHandler(Some(swallow), TRUE) == FALSE
            {
                return Err(format!("SetConsoleCtrlHandler: {}", std::io::Error::last_os_error()));
            }
        }

        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("cannot spawn the program: {e}"))?;

        let (ready_tx, ready_rx) = mpsc::channel::<()>();
        let stdout = drain(
            child.stdout.take().unwrap(),
            (readiness == "report").then(|| ready_tx.clone()),
            |line| line.starts_with('{'),
        );
        let stderr = drain(
            child.stderr.take().unwrap(),
            (readiness == "pause-marker").then(|| ready_tx.clone()),
            |line| line == PAUSE_MARKER,
        );
        drop(ready_tx);

        let mut error = None;
        if let Some(event) = event {
            if readiness != "none" && ready_rx.recv_timeout(READY_TIMEOUT).is_err() {
                error = Some("readiness signal not seen".to_owned());
            } else {
                // SAFETY: group 0 targets every process attached to this driver's own console.
                if unsafe { GenerateConsoleCtrlEvent(event, 0) } == FALSE {
                    error = Some(format!(
                        "GenerateConsoleCtrlEvent: {}",
                        std::io::Error::last_os_error()
                    ));
                }
            }
        }

        let deadline = Instant::now() + EXIT_TIMEOUT;
        let code = loop {
            match child.try_wait().map_err(|e| format!("wait: {e}"))? {
                Some(status) => break status.code(),
                None if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    error.get_or_insert_with(|| "the program did not exit in time".to_owned());
                    break None;
                }
                None => thread::sleep(Duration::from_millis(20)),
            }
        };
        let stdout = stdout.join().unwrap_or_default();
        let stderr = stderr.join().unwrap_or_default();

        let mut result =
            serde_json::json!({ "exit_code": code, "stdout": stdout, "stderr": stderr });
        if let Some(error) = error {
            result["error"] = error.into();
        }
        std::fs::write(&result_file, result.to_string())
            .map_err(|e| format!("cannot write the result: {e}"))
    }

    /// Reads a pipe to the end on its own thread, signalling once when `is_ready` matches a line.
    fn drain(
        pipe: impl Read + Send + 'static,
        ready: Option<mpsc::Sender<()>>,
        is_ready: fn(&str) -> bool,
    ) -> thread::JoinHandle<String> {
        thread::spawn(move || {
            let mut text = String::new();
            let mut ready = ready;
            let mut reader = BufReader::new(pipe);
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                if ready.is_some() && is_ready(line.trim_end()) {
                    let _ = ready.take().unwrap().send(());
                }
                text.push_str(&line);
                line.clear();
            }
            text
        })
    }
}
