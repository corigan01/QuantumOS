/*
  ____                 __               __ __                 __
 / __ \__ _____ ____  / /___ ____ _    / //_/__ _______  ___ / /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / ,< / -_) __/ _ \/ -_) /
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /_/|_|\__/_/ /_//_/\__/_/
  Part of the Quantum OS Kernel

Copyright 2022 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

use lazy_static::lazy_static;
use quantum_lib::debug_println;
use quantum_lib::x86_64::raw_cpu_io_port::{byte_in, byte_out};
use crate::clock::Time;
use spin::Mutex;

const CURRENT_CENTURY: u16 = 2000;

pub struct RealTimeClock {
    time: Time,
    time_zone_modification: i32
}

lazy_static! {
    static ref REAL_TIME_CLOCK: Mutex<RealTimeClock> = {
        Mutex::new(RealTimeClock::new())
    };
}

pub fn update_and_get_time() -> Time {
    REAL_TIME_CLOCK.lock().get_time_now()
}

pub fn get_time() -> Time {
    REAL_TIME_CLOCK.lock().time()
}

pub fn set_time_zone(zone: i32) {
    REAL_TIME_CLOCK.lock().set_time_zone(zone);
}

impl RealTimeClock {
    const CMOS_ADDRESS: u16 = 0x70;
    const CMOS_DATA: u16 = 0x71;

    pub fn new() -> Self {
        Self {
            time: Time::new(),
            time_zone_modification: 0,
        }
    }

    pub fn set_time_zone(&mut self, zone: i32) {
        self.time_zone_modification = zone;
    }

    /// # Time
    /// Does not update the time, just returns the last updated time!
    pub fn time(&self) -> Time {
        self.time
    }

    /// # Get Time Now
    /// Automatically runs `update` to update the time to the current time, but this can sometimes
    /// be slow if you want to get the time many times
    pub fn get_time_now(&mut self) -> Time {
        self.update_time();
        self.time()
    }

    pub fn update_time(&mut self) {

        // Avoid getting inconsistent data by waiting until its ready
        RealTimeClock::wait_for_update_flag();

        let mut current_rtc_time = Time::new();
        let mut last_rtc_time = Time::new();

        current_rtc_time.second = RealTimeClock::get_register(0x00) as u16;
        current_rtc_time.minute = RealTimeClock::get_register(0x02) as u16;
        current_rtc_time.hour   = RealTimeClock::get_register(0x04) as u16;
        current_rtc_time.day    = RealTimeClock::get_register(0x07) as u16;
        current_rtc_time.month  = RealTimeClock::get_register(0x08) as u16;
        current_rtc_time.year   = RealTimeClock::get_register(0x09) as u16;


        while current_rtc_time != last_rtc_time {
            last_rtc_time = current_rtc_time;

            current_rtc_time.second = RealTimeClock::get_register(0x00) as u16;
            current_rtc_time.minute = RealTimeClock::get_register(0x02) as u16;
            current_rtc_time.hour   = RealTimeClock::get_register(0x04) as u16;
            current_rtc_time.day    = RealTimeClock::get_register(0x07) as u16;
            current_rtc_time.month  = RealTimeClock::get_register(0x08) as u16;
            current_rtc_time.year   = RealTimeClock::get_register(0x09) as u16;
        }

        let reg_b = RealTimeClock::get_register(0x0B);

        if reg_b & 0x04 == 0 {
            current_rtc_time.second = (current_rtc_time.second  & 0x0F) + ((current_rtc_time.second / 16) * 10);
            current_rtc_time.minute = (current_rtc_time.minute  & 0x0F) + ((current_rtc_time.minute / 16) * 10);
            current_rtc_time.hour   = ((current_rtc_time.hour   & 0x0F) + (((current_rtc_time.hour & 0x70) / 16) * 10) ) | (current_rtc_time.hour & 0x80);
            current_rtc_time.day    = (current_rtc_time.day     & 0x0F) + ((current_rtc_time.day / 16) * 10);
            current_rtc_time.month  = (current_rtc_time.month   & 0x0F) + ((current_rtc_time.month / 16) * 10);
            current_rtc_time.year   = (current_rtc_time.year    & 0x0F) + ((current_rtc_time.year / 16) * 10) + CURRENT_CENTURY;
        }


        current_rtc_time.hour = (((current_rtc_time.hour & 0x7F) % 24) as i32 + self.time_zone_modification) as u16;
        current_rtc_time.hour = current_rtc_time.hour % 24;

        self.time = current_rtc_time;
    }

    fn get_register(reg: u8) -> u8 {
        unsafe { byte_out(RealTimeClock::CMOS_ADDRESS, reg); };
        unsafe { byte_in(RealTimeClock::CMOS_DATA) }
    }

    fn get_update_flag() -> u8 {
        unsafe { byte_out(RealTimeClock::CMOS_ADDRESS, 0x0A); };
        unsafe { byte_in(RealTimeClock::CMOS_DATA) & 0x80 }
    }

    fn wait_for_update_flag() {
        while RealTimeClock::get_update_flag() != 0 {}
    }
}

#[cfg(test)]
pub mod test_case {
    use crate::clock::rtc::{get_time, update_and_get_time};

    #[test_case]
    pub fn test_not_updated_time() {
        let time = get_time();

        let not_zero_time =
            time.second + time.minute + time.hour + time.day + time.month + time.year;

        assert_eq!(not_zero_time, 0);
    }

    #[test_case]
    pub fn updated_time() {
        let time = update_and_get_time();

        // TODO!
    }
}