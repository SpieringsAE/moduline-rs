use crate::ControllerSlot;
use embedded_hal::digital::OutputPin;
use std::{fs::File, os::unix::fs::FileExt};

pub struct SysfsOutput {
    fd: File,
}

impl SysfsOutput {
    pub fn new(slot: ControllerSlot) -> Result<Self, std::io::Error> {
        Ok(SysfsOutput {
            fd: std::fs::File::options()
                .read(true)
                .write(true)
                .open(format!(
                    "/sys/class/leds/ResetM-{}/brightness",
                    slot as u8 + 1
                ))?,
        })
    }
}

impl OutputPin for SysfsOutput {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.fd.write_at(b"0", 0)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.fd.write_at(b"1", 0)
    }

    fn set_state(&mut self, state: embedded_hal::digital::PinState) -> Result<(), Self::Error> {
        match state {
            embedded_hal::digital::PinState::Low => self.set_low(),
            embedded_hal::digital::PinState::High => self.set_high(),
        }
    }
}
