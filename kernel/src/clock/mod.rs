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

use core::fmt;
use core::fmt::Formatter;

pub mod rtc;

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Time {
    pub second: u16,
    pub minute: u16,
    pub hour: u16,
    pub day: u16,
    pub month: u16,
    pub year: u16,
}

impl Time {
    fn new() -> Self {
        Self {
            second: 0,
            minute: 0,
            hour: 0,
            day: 0,
            month: 0,
            year: 0,
        }
    }

    fn from_seconds(mut seconds: u64) -> Self {
        let year = seconds / 31_536_000;
        seconds -= year * 31_536_000;
        let month = seconds / 2_628_288;
        seconds -= month * 2_628_288;
        let day = seconds / 86400;
        seconds -= day * 86400;
        let hour = seconds / 3600;
        seconds -= hour * 3600;
        let minute = seconds / 60;
        seconds -= minute * 60;

        Self {
            second: seconds as u16,
            minute: minute as u16,
            hour: hour as u16,
            day: day as u16,
            month: month as u16,
            year: year as u16,
        }
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{:#02}:{:#02}:{:#02} {:#02}/{:#02}/{:#04}",
            self.hour, self.minute, self.second, self.month, self.day, self.year
        )
    }
}

#[cfg(test)]
pub mod test_case {
    use crate::clock::Time;

    #[test_case]
    pub fn test_case_from_seconds_test() {
        let time = Time::from_seconds(3602);

        assert_eq!(time.second, 2);
        assert_eq!(time.minute, 0);
        assert_eq!(time.hour, 1);
        assert_eq!(time.month, 0);
        assert_eq!(time.year, 0);

        let time = Time::from_seconds(32596368);

        assert_eq!(time.second, 48);
        assert_eq!(time.minute, 32);
        assert_eq!(time.hour, 6);
        assert_eq!(time.day, 12);
        assert_eq!(time.month, 0);
        assert_eq!(time.year, 1);
    }
}
