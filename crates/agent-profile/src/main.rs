//! The `agent-profile` binary. All behaviour lives in the library; this is the single exit path.

fn main() {
    std::process::exit(agent_profile::cli::run(std::env::args_os()));
}
