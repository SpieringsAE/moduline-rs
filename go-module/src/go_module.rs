use embedded_hal::{
    digital::{InputPin, OutputPin, PinState},
    spi::{Operation, SpiDevice},
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
    pub fn new(spi: SPI, reset: ResetPin, interrupt: InterruptPin) -> GoModule<SPI, ResetPin, InterruptPin> {
        GoModule { spi, reset, interrupt }
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
        slot: u8,
        direction: ModuleCommunicationDirection,
        module_id: u8,
        message_type: ModuleCommunicationType,
        message_index: u8,
        tx: &mut [u8],
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
        slot: u8,
        direction: ModuleCommunicationDirection,
        module_id: u8,
        message_type: ModuleCommunicationType,
        message_index: u8,
        tx: &mut [u8],
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

