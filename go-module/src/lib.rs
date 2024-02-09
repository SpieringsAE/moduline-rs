use embedded_hal::{
    digital::{InputPin, OutputPin, PinState},
    spi::{Operation, SpiDevice},
};

const BOOTMESSAGELENGTH: usize = 56;

pub struct GoModule<SPI, ResetPin, InterruptPin> {
    spi: SPI,
    reset: ResetPin,
    interrupt: InterruptPin,
}

impl<SPI, ResetPin, InterruptPin> GoModule<SPI, ResetPin, InterruptPin>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
{
    pub fn new(spi: SPI, reset: ResetPin, interrupt: InterruptPin) -> Self {
        GoModule {
            spi,
            reset,
            interrupt,
        }
    }

    pub fn escape_module_bootloader(&mut self) -> Result<[u8; BOOTMESSAGELENGTH], SPI::Error> {
        let mut tx = [0u8; BOOTMESSAGELENGTH];
        let mut rx = [0u8; BOOTMESSAGELENGTH];
        tx[0] = 19;
        tx[1] = (BOOTMESSAGELENGTH - 1) as u8;
        tx[2] = 19;
        tx[BOOTMESSAGELENGTH - 1] = GoModule::<SPI, ResetPin, InterruptPin>::module_checksum(&tx);
        self.spi
            .transaction(&mut [Operation::Transfer(&mut rx, &tx)])?;
        Ok(rx)
    }

    pub fn set_module_reset(&mut self, state: PinState) -> Result<(), ResetPin::Error> {
        self.reset.set_state(state)
    }

    pub fn get_module_interrupt(&mut self) -> Result<PinState, InterruptPin::Error> {
        if self.interrupt.is_high()? {
            Ok(PinState::High)
        } else {
            Ok(PinState::Low)
        }
    }

    pub fn module_checksum(data: &[u8]) -> u8 {
        let mut checksum: u8 = 0;
        for byte in data.iter() {
            checksum = checksum.wrapping_add(*byte);
        }
        checksum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linux_embedded_hal::{
        gpio_cdev::{Chip, LineRequestFlags},
        CdevPin, SpidevDevice,
    };

    #[test]
    fn it_works() {
        let dev = SpidevDevice::open("/dev/").unwrap();
        let mut chip = Chip::new("/dev/gpio0").unwrap();
        let reset_line = chip.get_line(0).unwrap();
        let reset_linehandle = reset_line
            .request(LineRequestFlags::OUTPUT, 0, "module-reset")
            .unwrap();
        let reset = CdevPin::new(reset_linehandle).unwrap();
        let interrupt_line = chip.get_line(1).unwrap();
        let interrupt_line_handle = interrupt_line
            .request(LineRequestFlags::INPUT, 0, "module-interrupt")
            .unwrap();
        let interrupt = CdevPin::new(interrupt_line_handle).unwrap();
        let module = GoModule::new(dev, reset, interrupt);
    }
}
