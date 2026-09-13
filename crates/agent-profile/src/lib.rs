//! `agent-profile` — select and launch profiles for multiple coding agents.
//!
//! The authoritative design is `agent-profile-implementation-spec-v3.md` at the repository root.
//! Each module below is one layer of the spec §4 architecture. SP0 (the scaffold) creates them
//! empty; later sub-projects fill them in.
//!
//! > `agent-profile` owns profile selection. The coding agent owns authentication and
//! > agent-specific configuration. (spec §1)

pub mod adapter;
pub mod cli;
pub mod config;
pub mod launch;
pub mod name;
pub mod output;
pub mod repo;
pub mod resolve;
