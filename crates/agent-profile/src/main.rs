// Copyright (c) 2026 Costas Kirgoussios
//
// This software is licensed under the PolyForm Noncommercial License 1.0.0.
// You may obtain a copy of the License at:
// https://polyformproject.org/licenses/noncommercial/1.0.0/

//! The `agent-profile` binary. All behaviour lives in the library; this is the single exit path.

fn main() {
    std::process::exit(agent_profile::cli::run(std::env::args_os()));
}
