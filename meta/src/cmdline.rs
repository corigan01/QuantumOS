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

    /// Print all interrupts to std out
    #[arg(long = "log-int", default_value_t = false)]
    pub log_interrupts: bool,

    /// Print std out to command-line
    #[arg(long = "nographic", default_value_t = false)]
    pub no_graphic: bool,
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
