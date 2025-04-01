#![no_std]
#![allow(unused)]

mod blocking;
mod register;



#[doc(hidden)]
pub mod prelude {
    pub use embedded_io::{Read as _, Write as _,ReadReady as _, WriteReady as _};
    pub use embedded_hal_nb::{serial::Read as _,serial::Write as _ };
}


pub use crate::blocking::BlockingUart;
pub use crate::register::*;

/// Configuration struct for UART settings.
///
/// This struct contains all configurable parameters for the UART interface.
/// Including divisor, parity mode, stop bits and word length settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// The divisor value for baud rate generation.
    pub divisor: u16,
    /// The parity checking mode.
    pub parity_mode: ParityMode,
    /// Number of stop bits.
    pub stop_bits: StopBits,
    /// Length of data words.
    pub word_length: WordLength,
}

impl Config {
    /// Creates a new Config with default settings.
    ///
    /// Default settings are:
    /// - No divisor.
    /// - No parity.
    /// - 1 stop bit.
    /// - 8 bits word length.
    pub fn new() -> Self {
        Self {
            divisor: 0,
            parity_mode: ParityMode::None,
            stop_bits: StopBits::Bit1,
            word_length: WordLength::Bits8,
        }
    }

    /// Sets the divisor value.
    pub fn set_divisor(mut self, divisor: u16) -> Self {
        self.divisor = divisor;
        self
    }

    /// Sets the parity mode.
    pub fn set_parity_mode(mut self, parity_mode: ParityMode) -> Self {
        self.parity_mode = parity_mode;
        self
    }

    /// Sets the number of stop bits.
    pub fn set_stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    /// Sets the word length.
    pub fn set_word_length(mut self, word_length: WordLength) -> Self {
        self.word_length = word_length;
        self
    }
}

/// Represents different parity checking modes for UART communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityMode {
    /// No parity checking.
    None,
    /// Odd parity checking.
    Odd,
    /// Even parity checking.
    Even,
    /// Force parity bit high.
    High,
    /// Force parity bit low.
    Low,
}

/// Gets the current divisor value from UART registers.
pub(crate) fn divisor(uart: &RegisterBlock) -> u16 {
    let lcr = uart.lcr.read();
    unsafe {
        uart.lcr.write(lcr.enable_divisor_latch_access());
    }
    let dll = uart.rbr_thr_dll.read().divisor_latch_low_byte();
    let dlh = uart.ier_dlh.read().divisor_latch_high_byte();
    u16::from_le_bytes([dll, dlh])
}

/// Sets the divisor value in UART registers.
pub(crate) fn set_divisor(uart: &RegisterBlock, divisor: u16) {
    let lcr = uart.lcr.read();
    let [divisor_low, divisor_high] = divisor.to_le_bytes();
    unsafe {
        uart.lcr.write(lcr.enable_divisor_latch_access());
    }
    let dll = uart
        .rbr_thr_dll
        .read()
        .set_divisor_latch_low_byte(divisor_low);
    let dlh = uart
        .ier_dlh
        .read()
        .set_divisor_latch_high_byte(divisor_high);
    unsafe {
        uart.rbr_thr_dll.write(dll);
        uart.ier_dlh.write(dlh);
        uart.lcr.write(lcr);
    }
}

/// Gets the current parity mode from UART registers.
pub(crate) fn parity_mode(uart: &RegisterBlock) -> ParityMode {
    let lcr = uart.lcr.read();
    let flags = (
        lcr.is_parity_enabled(),
        lcr.parity(),
        lcr.is_stick_parity_enabled(),
    );
    match flags {
        (false, _, _) => ParityMode::None,
        (true, Parity::Even, false) => ParityMode::Even,
        (true, Parity::Odd, false) => ParityMode::Odd,
        (true, Parity::Odd, true) => ParityMode::High,
        (true, Parity::Even, true) => ParityMode::Low,
    }
}

/// Sets the parity mode in UART registers.
pub(crate) fn set_parity_mode(uart: &RegisterBlock, parity: ParityMode) {
    let lcr = uart.lcr.read();
    let lcr = match parity {
        ParityMode::None => lcr.disable_parity(),
        ParityMode::Odd => lcr
            .enable_parity()
            .disable_stick_parity()
            .set_parity(Parity::Odd),
        ParityMode::Even => lcr
            .enable_parity()
            .disable_stick_parity()
            .set_parity(Parity::Even),
        ParityMode::High => lcr
            .enable_parity()
            .enable_stick_parity()
            .set_parity(Parity::Odd),
        ParityMode::Low => lcr
            .enable_parity()
            .enable_stick_parity()
            .set_parity(Parity::Even),
    };
    unsafe {
        uart.lcr.write(lcr);
    }
}

/// Gets the current stop bits setting from UART registers.
pub(crate) fn stop_bits(uart: &RegisterBlock) -> StopBits {
    uart.lcr.read().stop_bits()
}

/// Sets the stop bits in UART registers.
pub(crate) fn set_stop_bits(uart: &RegisterBlock, stop_bits: StopBits) {
    let lcr = uart.lcr.read().set_stop_bits(stop_bits);
    unsafe {
        uart.lcr.write(lcr);
    }
}

/// Gets the current word length from UART registers.
pub(crate) fn word_length(uart: &RegisterBlock) -> WordLength {
    uart.lcr.read().word_length()
}

/// Sets the word length in UART registers.
pub(crate) fn set_word_length(uart: &RegisterBlock, word_length: WordLength) {
    let lcr = uart.lcr.read().set_word_length(word_length);
    unsafe {
        uart.lcr.write(lcr);
    }
}

/// Checks if the UART is ready to read data.
pub(crate) fn read_ready(uart: &RegisterBlock) -> bool {
    uart.lsr.read().is_data_ready()
}

/// Checks if the UART is ready to write data.
pub(crate) fn write_ready(uart: &RegisterBlock) -> bool {
    uart.lsr.read().is_transmitter_fifo_empty()
}
