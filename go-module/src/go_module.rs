use embedded_hal::{
    digital::{InputPin, OutputPin, PinState},
    spi::{Operation, SpiDevice},
};
use go_mainboard::{ControllerSlot, ControllerType};
#[cfg(feature = "std")]
use linux_embedded_hal::{
    gpio_cdev::{Chip, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, SpidevDevice,
};

const BOOTMESSAGELENGTH: usize = 56;

///Internal Struct that holds the driver for different types of modules
pub struct GoModule<SPI, ResetPin, InterruptPin> {
    spi: SPI,
    reset: ResetPin,
    interrupt: InterruptPin,
}

#[derive(Copy, Clone, Debug)]
pub enum GoModuleError<SPI, ResetPin, InterruptPin> {
    SPI(SPI),
    ResetPin(ResetPin),
    InterruptPin(InterruptPin),
    ModuleSetupError(ModuleSetupError),
    CommunicationError(CommunicationError),
}

#[derive(Debug, Clone, Copy)]
pub enum ModuleSetupError {
    InterruptPin,
    ResetPin,
    Spi,
}

#[derive(Debug, Clone, Copy)]
pub enum CommunicationError {
    ModuleUnavailable,
    ChecksumIncorrect,
}

#[repr(u8)]
pub enum ModuleCommunicationDirection {
    ToModule = 1,
    FromModule,
}

#[repr(u8)]
pub enum ModuleCommunicationType {
    ModuleId = 1,
    Configuration,
    Data,
    Feedback,
}

impl<SPI, ResetPin, InterruptPin> GoModule<SPI, ResetPin, InterruptPin>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
{
    #[cfg(feature = "std")]
    pub fn new(
        controller_type: ControllerType,
        slot: ControllerSlot,
    ) -> Result<Self, ModuleSetupError> {
        extern crate std;
        let (interrupt, reset, spi) = match controller_type {
            ControllerType::ModulineIV(_) => match slot {
                ControllerSlot::Slot1 => {
                    (
                        get_module_interrupt("gpiochip0", 6, &slot)?,
                        get_module_reset("gpiochip0", 7, &slot)?,
                        get_module_spi("/dev/spidev1.0")?,
                    )

                }
                ControllerSlot::Slot2 => {
                    (
                        get_module_interrupt("gpiochip4", 20, &slot)?,
                        get_module_reset("gpiochip2", 10, &slot)?,
                        get_module_spi("/dev/spidev1.1")?,
                    )
                }
                ControllerSlot::Slot3 => {
                    (
                        get_module_interrupt("gpiochip0", 7, &slot)?,
                        get_module_reset("gpiochip0", 4, &slot)?,
                        get_module_spi("/dev/spidev2.0")?,
                    )
                }
                ControllerSlot::Slot4 => {
                    (
                        get_module_interrupt("gpiochip4", 21, &slot)?,
                        get_module_reset("gpiochip0", 2, &slot)?,
                        get_module_spi("/dev/spidev2.1")?,
                    )
                }
                ControllerSlot::Slot5 => {
                    (
                        get_module_interrupt("gpiochip4", 1, &slot)?,
                        get_module_reset("gpiochip3", 24, &slot)?,
                        get_module_spi("/dev/spidev2.2")?,
                    )
                }
                ControllerSlot::Slot6 => {
                    (
                        get_module_interrupt("gpiochip3", 26, &slot)?,
                        get_module_reset("gpiochip3", 27, &slot)?,
                        get_module_spi("/dev/spidev2.3")?,
                    )
                }
                ControllerSlot::Slot7 => {
                    (
                        get_module_interrupt("gpiochip2", 19, &slot)?,
                        get_module_reset("gpiochip2", 24, &slot)?,
                        get_module_spi("/dev/spidev0.0")?,
                    )
                }
                ControllerSlot::Slot8 => {
                    (
                        get_module_interrupt("gpiochip2", 22, &slot)?,
                        get_module_reset("gpiochip2", 20, &slot)?,
                        get_module_spi("/dev/spidev0.1")?,
                    )
                }
            },
            _ => todo!("implement all the hardware"),
        };
        Ok(GoModule { spi, reset, interrupt })
    }

    #[cfg(not(feature = "std"))]
    pub fn new(controller_type: ControllerType, slot: ControllerSlot, dp: &mut Peripherals) -> Result<Self, ModuleSetupError> {
        let (interrupt, reset, spi) = match ControllerType {
            ControllerType::ModulineII(_) => {
                match slot {
                    ControllerSlot::Slot1 => {
                        todo!("figure out Moduline 2 stuff")
                    }
                    _=> todo!("more slots")
                }
            }
        };
        Ok(GoModule { spi, reset, interrupt })
    }

    pub fn escape_module_bootloader(
        &mut self,
    ) -> Result<
        [u8; BOOTMESSAGELENGTH],
        GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>,
    > {
        let mut tx = [0u8; BOOTMESSAGELENGTH];
        let mut rx = [0u8; BOOTMESSAGELENGTH];
        tx[0] = 19;
        tx[1] = (BOOTMESSAGELENGTH - 1) as u8;
        tx[2] = 19;
        tx[BOOTMESSAGELENGTH - 1] = module_checksum(&tx);
        self.spi
            .transaction(&mut [Operation::Transfer(&mut rx, &tx)])
            .map_err(GoModuleError::SPI)?;
        Ok(rx)
    }

    pub fn send_spi(
        &mut self,
        slot: ControllerSlot,
        direction: ModuleCommunicationDirection,
        module_id: u8,
        message_type: ModuleCommunicationType,
        message_index: u8,
        tx: &[u8],
        delay_us: u16,
    ) -> Result<(), GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
        tx[0] = slot as u8 + 1;
        tx[1] = tx.len() as u8 - 1;
        tx[2] = direction as u8;
        tx[3] = module_id;
        tx[4] = message_type as u8;
        tx[5] = message_index;
        tx[tx.len() - 1] = module_checksum(tx);

        let mut transactions = [
            Operation::DelayNs(delay_us as u32 * 1000),
            Operation::Write(tx),
        ];
        if self
            .interrupt
            .is_high()
            .map_err(GoModuleError::InterruptPin)?
        {
            self.spi
                .transaction(&mut transactions)
                .map_err(GoModuleError::SPI)?;
            Ok(())
        } else {
            Err(GoModuleError::CommunicationError(
                CommunicationError::ModuleUnavailable,
            ))
        }
    }

    pub fn send_receive_spi(
        &mut self,
        slot: ControllerSlot,
        direction: ModuleCommunicationDirection,
        module_id: u8,
        message_type: ModuleCommunicationType,
        message_index: u8,
        tx: &[u8],
        rx: &mut [u8],
        delay_us: u16,
    ) -> Result<(), GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
        tx[0] = slot as u8 + 1;
        tx[1] = tx.len() as u8 - 1;
        tx[2] = direction as u8;
        tx[3] = module_id;
        tx[4] = message_type as u8;
        tx[5] = message_index;
        tx[tx.len() - 1] = module_checksum(tx);
        rx[0] = 0;
        rx[rx.len() - 1] = 0;

        let mut transactions = [
            Operation::DelayNs(delay_us as u32 * 100),
            Operation::Transfer(rx, tx),
        ];
        if self
            .interrupt
            .is_high()
            .map_err(GoModuleError::InterruptPin)?
        {
            self.spi
                .transaction(&mut transactions)
                .map_err(GoModuleError::SPI)?;
            if module_checksum(&rx) == rx[rx.len() - 1]
                && rx[1] == rx.len() as u8 - 1
            {
                Ok(())
            } else {
                Err(GoModuleError::CommunicationError(
                    CommunicationError::ChecksumIncorrect,
                ))
            }
        } else {
            Err(GoModuleError::CommunicationError(
                CommunicationError::ModuleUnavailable,
            ))
        }
    }

    pub fn set_module_reset(
        &mut self,
        state: PinState,
    ) -> Result<(), GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
        self.reset.set_state(state).map_err(GoModuleError::ResetPin)
    }

    pub fn get_module_interrupt_state(
        &mut self,
    ) -> Result<PinState, GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
        if self
            .interrupt
            .is_high()
            .map_err(GoModuleError::InterruptPin)?
        {
            Ok(PinState::High)
        } else {
            Ok(PinState::Low)
        }
    }
}

pub fn module_checksum(data: &[u8]) -> u8 {
        let mut checksum: u8 = 0;
        for byte in data.iter() {
            checksum = checksum.wrapping_add(*byte);
        }
        checksum
    }

#[cfg(feature = "std")]
fn get_module_interrupt(
    chip: &str,
    line: u32,
    slot: &ControllerSlot,
) -> Result<CdevPin, ModuleSetupError> {
    extern crate std;
    let mut chip = Chip::new(chip).map_err(|_| ModuleSetupError::InterruptPin)?;
    let line = chip
        .get_line(6)
        .map_err(|_| ModuleSetupError::InterruptPin)?;
    let line_handle = line
        .request(
            LineRequestFlags::INPUT,
            0,
            std::format!("slot {} module interrupt", *slot as u8 + 1).as_str(),
        )
        .map_err(|_| ModuleSetupError::InterruptPin)?;
    Ok(CdevPin::new(line_handle).map_err(|_| ModuleSetupError::InterruptPin)?)
}

#[cfg(feature = "std")]
fn get_module_reset(
    chip: &str,
    line: u32,
    slot: &ControllerSlot,
) -> Result<CdevPin, ModuleSetupError> {
    extern crate std;
    let mut chip = Chip::new(chip).map_err(|_| ModuleSetupError::InterruptPin)?;
    let line = chip.get_line(7).map_err(|_| ModuleSetupError::ResetPin)?;
    let line_handle = line
        .request(
            LineRequestFlags::OUTPUT,
            0,
            std::format!("slot {} module reset", *slot as u8 + 1).as_str(),
        )
        .map_err(|_| ModuleSetupError::ResetPin)?;
    Ok(CdevPin::new(line_handle).map_err(|_| ModuleSetupError::ResetPin)?)
}

#[cfg(feature = "std")]
fn get_module_spi(spidev: &str) -> Result<SpidevDevice, ModuleSetupError> {
    extern crate std;
    let mut spi = SpidevDevice::open(spidev).map_err(|_| ModuleSetupError::Spi)?;
    let spi_opts = SpidevOptions {
        bits_per_word: Some(8),
        max_speed_hz: Some(2_000_000),
        lsb_first: None,
        spi_mode: Some(SpiModeFlags::SPI_MODE_0),
    };
    spi.configure(&spi_opts)
        .map_err(|_| ModuleSetupError::Spi)?;
    Ok(spi)
}
