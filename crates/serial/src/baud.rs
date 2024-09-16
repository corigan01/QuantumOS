/// # Serial Baud
/// Set a supported serial baud rate for serial comms.
#[derive(Clone, Copy, Debug)]
pub enum SerialBaud {
    Baud115200,
    Baud57600,
    Baud38400,
    Baud19200,
    Baud14400,
    Baud9600,
    Baud4800,
    Baud2400,
    Baud1200,
    Baud600,
    Baud300,
}

impl SerialBaud {
    /// # Get Divisor
    /// Get the divisor for serial baud register's clock. This will
    /// get the divisor setting needed to get to one of the set baud
    /// rates.
    pub const fn get_divisor(self) -> u16 {
        match self {
            Self::Baud115200 => 1,
            Self::Baud57600 => 2,
            Self::Baud38400 => 3,
            Self::Baud19200 => 6,
            Self::Baud14400 => 8,
            Self::Baud9600 => 12,
            Self::Baud4800 => 24,
            Self::Baud2400 => 48,
            Self::Baud1200 => 96,
            Self::Baud600 => 192,
            Self::Baud300 => 348,
        }
    }
}
