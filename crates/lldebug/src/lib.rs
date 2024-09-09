#![no_std]

use core::fmt::Write;

/// # Instance String
/// The name used as a header for each newline printed. Useful for describing where
/// in the operating system you are executing.
static mut INSTANCE_STRING: Option<&'static str> = None;

/// # Set Crate Nametag
/// This function will tell LLDebug who you are, and will tell the user that this
/// output is coming from your crate.
pub fn set_crate_nametag(name: &'static str) {
    unsafe { INSTANCE_STRING = Some(name) };
}

/// # Outlet Ref
/// The type of a rust `core::fmt::Write`-able struct for sending out debug
/// strings to the user.
pub type OutletRef = &'static mut (dyn Write + Sync + Send);

/// # Debug Stream
/// Debug stream describes an output to which debug info can be posted. For
/// example, you can attach a serial debug and vga debug stream and have all
/// debug output routed to both.
pub struct DebugStream {
    stream_name: &'static str,
    outlet: OutletRef,
}

impl DebugStream {
    /// # New
    /// Create a new debug stream with stream name and outlet provider.
    pub fn new(stream_name: &'static str, outlet: OutletRef) -> Self {
        Self {
            stream_name,
            outlet,
        }
    }
}

impl Write for DebugStream {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        todo!()
    }
}
