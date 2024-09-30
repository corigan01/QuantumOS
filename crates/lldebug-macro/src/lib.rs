/*
// make_debug! can be lazy, so the first `println!()` call will
// construct the output streams.
make_debug! {
    #[debug(Serial)]
    fn() -> Option<Serial> { ... }

    #[debug(ScreenBuffer)]
    // This could be in-case there is no Default implemented
    fn() -> Option<Serial> { ... }
}

->


mod _debug {
    static mut DEBUG_OUTPUT_STREAM_SERIAL: Mutex<LazyCell<Option<Serial>>>
        = Mutex::new(LazyCell::new(|| { init_macro(); ... }));

    static mut DEBUG_OUTPUT_STREAM_SCREEN_BUFFER: Mutex<LazyCell<Option<ScreenBuffer>>>
        = Mutex::new(LazyCell::new(|| { init_macro();  ... }));

    static mut HAS_ALREADY_INIT: AtomicBool = AtomicBool::new(false);
    fn init_macro() {
        // Check if init is already done
        if unsafe { HAS_ALREADY_INIT.load(Ordering::Acquire) } {
            return;
        }

        // Ensure no one else can init
        unsafe { HAS_ALREADY_INIT.store(true, Ordering::Store) };

        lldebug::set_output_fn(GLOBAL_OUTPUT_PTR);
    }

    const GLOBAL_OUTPUT_PTR: fn(fmt::Arguments) -> fmt::Result = global_output;
    fn global_output(fmt: fmt::Arguments) -> fmt::Result {
        // List of all output streams

        /*Make sure is Some(x)*/ DEBUG_OUTPUT_STREAM_SERIAL.write_fmt(fmt)?;
        /*Make sure is Some(x)*/ DEBUG_OUTPUT_STREAM_SCREEN_BUFFER.write_fmt(fmt)?;

        Ok(())
    }
}

*/
