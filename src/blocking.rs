#![allow(unused)]

use crate::register::RegisterBlock;
use crate::{
    Config, RbrThrDll, divisor, parity_mode, read_ready, set_divisor, set_parity_mode,
    set_stop_bits, set_word_length, stop_bits, word_length, write_ready,
};
use embedded_hal_nb::nb;
use embedded_io::ErrorType;
use core::ops::Deref;

/// Reads data from UART in a blocking manner.
///
/// This function attempts to read data from the UART into the provided buffer.
/// It will read as much data as possible until either the buffer is full or no more data is available.
/// Returns the number of bytes actually read.
fn blocking_read(uart: &RegisterBlock, buf: &mut [u8]) -> usize {
    let mut count = 0_usize;
    for ch in buf {
        if uart.lsr.read().is_data_ready() {
            *ch = uart.rbr_thr_dll.read().receiver_data();
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Writes data to UART in a blocking manner.
///
/// This function attempts to write data from the provided buffer to the UART.
/// It will write as much data as possible until either all data is written or the FIFO becomes full.
/// Returns the number of bytes actually written.
fn blocking_write(uart: &RegisterBlock, buf: &[u8]) -> usize {
    let mut count = 0_usize;
    for ch in buf {
        if uart.lsr.read().is_transmitter_fifo_empty() {
            let thr = RbrThrDll::default().set_transmitter_data(*ch);
            unsafe {
                uart.rbr_thr_dll.write(thr);
            }
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Flushes the UART transmitter by waiting until all data has been sent.
///
/// This function blocks until the transmitter is completely empty.
fn blocking_flash(uart: &RegisterBlock) {
    while !uart.lsr.read().is_transmitter_empty() {
        core::hint::spin_loop();
    }
}

/// A wrapper struct for UART that provides blocking operations.
///
/// This struct implements blocking read and write operations for UART communication.
pub struct BlockingUart<UART> {
    uart: UART,
}

impl<UART: Deref<Target = RegisterBlock>> BlockingUart<UART> {
    /// Creates a new BlockingUart instance with the specified configuration.
    ///
    /// This function initializes the UART with the provided configuration parameters.
    /// Returns a new BlockingUart instance.
    pub fn new(uart: UART, config: Config) -> Self {
        if let Some(divisor) = config.divisor {
            set_divisor(&uart, divisor);
        }
        set_parity_mode(&uart, config.parity_mode);
        set_stop_bits(&uart, config.stop_bits);
        set_word_length(&uart, config.word_length);

        BlockingUart { uart }
    }

    /// Returns the current configuration of the UART.
    ///
    /// This function reads all configuration parameters from the UART registers and returns them as a Config struct.
    pub fn config(&self) -> Config {
        let divisor = Some(divisor(&self.uart));
        let parity_mode = parity_mode(&self.uart);
        let stop_bits = stop_bits(&self.uart);
        let word_length = word_length(&self.uart);
        Config {
            divisor,
            parity_mode,
            stop_bits,
            word_length,
        }
    }

    /// Reads data from the UART into the provided buffer.
    ///
    /// Returns the number of bytes actually read.
    pub fn read(&self, buf: &mut [u8]) -> usize {
        blocking_read(&self.uart, buf)
    }

    /// Writes data from the provided buffer to the UART.
    ///
    /// Returns the number of bytes actually written.
    pub fn write(&mut self, buf: &[u8]) -> usize {
        blocking_write(&self.uart, buf)
    }

    /// Flushes the UART transmitter.
    ///
    /// This function ensures all data has been transmitted before returning.
    pub fn flash(&self) {
        blocking_flash(&self.uart)
    }
}

impl<UART: Deref<Target = RegisterBlock>> ErrorType for BlockingUart<UART> {
    type Error = core::convert::Infallible;
}

impl<UART: Deref<Target = RegisterBlock>> embedded_io::Read for BlockingUart<UART> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(blocking_read(&self.uart, buf))
    }
}

impl<UART: Deref<Target = RegisterBlock>> embedded_io::Write for BlockingUart<UART> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(blocking_write(&self.uart, buf))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        blocking_flash(&self.uart);
        Ok(())
    }
}

impl<UART: Deref<Target = RegisterBlock>> embedded_io::ReadReady for BlockingUart<UART> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(read_ready(&self.uart))
    }
}

impl<UART: Deref<Target = RegisterBlock>> embedded_io::WriteReady for BlockingUart<UART> {
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(write_ready(&self.uart))
    }
}

impl<UART: Deref<Target = RegisterBlock>> embedded_hal_nb::serial::ErrorType
    for BlockingUart<UART>
{
    type Error = core::convert::Infallible;
}

impl<UART: Deref<Target = RegisterBlock>> embedded_hal_nb::serial::Read for BlockingUart<UART> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut buf = [0];
        let len = blocking_read(&self.uart, &mut buf);
        match len {
            0 => Err(nb::Error::WouldBlock),
            _ => Ok(buf[0]),
        }
    }
}

impl<UART: Deref<Target = RegisterBlock>> embedded_hal_nb::serial::Write for BlockingUart<UART> {
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        let len = blocking_write(&self.uart, &[word]);
        match len {
            0 => Err(nb::Error::WouldBlock),
            _ => Ok(()),
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        match self.uart.lsr.read().is_transmitter_empty() {
            true => Ok(()),
            false => Err(nb::Error::WouldBlock),
        }
    }
}
