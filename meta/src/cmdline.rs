use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
/// Build System for Quantum OS
pub struct CommandLine {
    /// Build Option
    #[command(subcommand)]
    pub option: Option<TaskOption>,

    /// Enable Qemu's KVM
    #[arg(long, default_value_t = false)]
    pub enable_kvm: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TaskOption {
    /// Build Quantum OS
    Build,
    /// Run + Build Quantum OS
    Run,
    /// Clean up all build artifacts
    Clean,
}
