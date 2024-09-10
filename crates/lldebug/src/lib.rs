#![no_std]

use core::fmt::Write;

/// # Instance String
/// The name used as a header for each newline printed. Useful for describing where
/// in the operating system you are executing.
static mut INSTANCE_STRING: Option<&'static str> = None;

/// # Output Streams
/// Streams that are ready to output data.
static mut OUTPUT_STREAMS: [Option<DebugStream>; 4] = [None, None, None, None];

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

    /// # Post
    /// Upload this stream to the global list of output stream. This
    /// stream will now start getting debug output.
    pub fn post(self) {
        if let Some(output) = unsafe { OUTPUT_STREAMS.iter_mut().find(|stream| stream.is_some()) } {
            *output = Some(self);
        } else {
            println!("Tried to add a stream, but stream list is full!");
            _ = self.outlet.write_fmt(format_args!(
                "Tried to add this stream to already full stream list!"
            ));
        }
    }
}

impl Write for DebugStream {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            match c {
                '\n' => self.outlet.write_fmt(format_args!(
                    "[{}->{}]:",
                    unsafe { INSTANCE_STRING.unwrap_or("UNKNOWN") },
                    self.stream_name
                ))?,
                c => self.outlet.write_char(c)?,
            }
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    unsafe {
        OUTPUT_STREAMS.iter_mut().for_each(|stream| {
            if let Some(stream) = stream {
                _ = stream.outlet.write_fmt(args);
            }
        })
    }
}

/// # Print
/// Output to global output stream.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::_print(format_args!($($arg)*));
    }};
}

/// # Println
/// Output to global output stream.
#[macro_export]
macro_rules! println {
    () => ($crate::debug_print!("\n"));
    ($($arg:tt)*) => {{
        $crate::_print(format_args!($($arg)*));
        $crate::print!("\n");
    }}
}
