///Internal Struct that holds the driver for different types of modules
pub struct GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay> {
    spi: SPI,
    reset: ResetPin,
    interrupt: InterruptPin,
    delay: Delay,
    slot: u8,
}

pub struct GoModule<SPI, ResetPin, InterruptPin, Delay> {
    spi: SPI,
    reset: ResetPin,
    interrupt: InterruptPin,
    pub delay: Delay,
    slot: u8,
}

const BOOTMESSAGELENGTH: usize = 46;

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
    UnableToSerDe,
}

#[repr(u8)]
pub enum ModuleCommunicationDirection {
    ToModule = 1,
    FromModule,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ModuleCommunicationType {
    ModuleId = 1,
    Configuration,
    Data,
    Feedback,
}
#[cfg(not(feature = "async"))]
pub mod go_module {

    use core::usize;

    use crate::GoModuleUnknown;

    use super::{
        module_checksum, CommunicationError, GoModule, GoModuleError, ModuleCommunicationDirection,
        ModuleCommunicationType, BOOTMESSAGELENGTH,
    };
    use embedded_hal::delay::DelayNs;
    use embedded_hal::digital::{InputPin, OutputPin, PinState};
    use embedded_hal::spi::{Operation, SpiDevice};

    impl<SPI, ResetPin, InterruptPin, Delay> GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay>
    where
        SPI: SpiDevice,
        ResetPin: OutputPin,
        InterruptPin: InputPin,
        Delay: DelayNs,
    {
        pub fn new(
            spi: SPI,
            reset: ResetPin,
            interrupt: InterruptPin,
            delay: Delay,
            slot: u8,
        ) -> GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay> {
            debug_assert!(slot > 0, "slot needs to be larger than 0");
            GoModuleUnknown {
                spi,
                reset,
                interrupt,
                delay,
                slot,
            }
        }

        pub fn module_reset(
            mut self,
        ) -> Result<
            GoModule<SPI, ResetPin, InterruptPin, Delay>,
            GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay>,
        > {
            if self.reset.set_state(PinState::Low).is_err() {
                return Err(self);
            }
            self.delay.delay_ms(100);
            if self.reset.set_state(PinState::High).is_err() {
                return Err(self);
            }
            self.delay.delay_ms(100);
            Ok(GoModule {
                spi: self.spi,
                reset: self.reset,
                interrupt: self.interrupt,
                delay: self.delay,
                slot: self.slot,
            })
        }
    }

    impl<SPI, ResetPin, InterruptPin, Delay> GoModule<SPI, ResetPin, InterruptPin, Delay>
    where
        SPI: SpiDevice,
        ResetPin: OutputPin,
        InterruptPin: InputPin,
        Delay: DelayNs,
    {
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
            tx[BOOTMESSAGELENGTH - 1] = module_checksum(&tx, BOOTMESSAGELENGTH);
            self.spi
                .transaction(&mut [Operation::Transfer(&mut rx, &tx)])
                .map_err(GoModuleError::SPI)?;
            Ok(rx)
        }

        pub fn send_spi(
            &mut self,
            direction: ModuleCommunicationDirection,
            module_id: u8,
            message_type: ModuleCommunicationType,
            message_index: u8,
            tx: &mut [u8],
            len: usize,
            delay_us: u32,
        ) -> Result<(), GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
            debug_assert!(
                len <= tx.len(),
                "len cannot be longer than the actual buffer length"
            );
            tx[0] = self.slot;
            tx[1] = len as u8 - 1;
            tx[2] = direction as u8;
            tx[3] = module_id;
            tx[4] = message_type as u8;
            tx[5] = message_index;
            tx[len - 1] = module_checksum(tx, len);

            let mut transactions = [Operation::Write(tx)];
            //            if self
            //              .interrupt
            //            .is_low()
            //          .map_err(GoModuleError::InterruptPin)?
            //    {
            self.delay.delay_us(delay_us);
            self.spi
                .transaction(&mut transactions)
                .map_err(GoModuleError::SPI)?;
            Ok(())
            //            } else {
            //              Err(GoModuleError::CommunicationError(
            //                CommunicationError::ModuleUnavailable,
            //          ))
            //    }
        }

        pub fn send_receive_spi(
            &mut self,
            direction: ModuleCommunicationDirection,
            module_id: u8,
            message_type: ModuleCommunicationType,
            message_index: u8,
            tx: &mut [u8],
            rx: &mut [u8],
            len: usize,
            delay_us: u32,
        ) -> Result<(), GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
            debug_assert!(
                tx.len() == rx.len(),
                "receive and transmit buffer must have equal length"
            );
            debug_assert!(
                len <= tx.len(),
                "len cannot be longer than the actual buffer length"
            );
            tx[0] = self.slot;
            tx[1] = len as u8 - 1;
            tx[2] = direction as u8;
            tx[3] = module_id;
            tx[4] = message_type as u8;
            tx[5] = message_index;
            tx[len - 1] = module_checksum(tx, len);

            let mut transactions = [Operation::Transfer(rx, tx)];
            //            if self
            //              .interrupt
            //            .is_low()
            //          .map_err(GoModuleError::InterruptPin)?
            //    {
            self.delay.delay_us(delay_us);
            self.spi
                .transaction(&mut transactions)
                .map_err(GoModuleError::SPI)?;
            if module_checksum(&rx, len) == rx[len - 1] && rx[1] as usize == len {
                Ok(())
            } else {
                Err(GoModuleError::CommunicationError(
                    CommunicationError::ChecksumIncorrect,
                ))
            }
            //            } else {
            //              Err(GoModuleError::CommunicationError(
            //                CommunicationError::ModuleUnavailable,
            //          ))
            //    }
        }

        pub fn get_module_interrupt_state(
            &mut self,
        ) -> Result<PinState, GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>>
        {
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

        pub fn degrade(self) -> GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay> {
            GoModuleUnknown {
                spi: self.spi,
                reset: self.reset,
                interrupt: self.interrupt,
                delay: self.delay,
                slot: self.slot,
            }
        }
    }
}

#[cfg(feature = "async")]
pub mod go_module_async {
    use crate::GoModuleUnknown;

    use super::{
        module_checksum, CommunicationError, GoModule, GoModuleError, ModuleCommunicationDirection,
        ModuleCommunicationType, BOOTMESSAGELENGTH,
    };
    use embedded_hal::digital::{InputPin, OutputPin, PinState};

    use embedded_hal_async::delay::DelayNs;
    #[cfg(feature = "async")]
    use embedded_hal_async::{
        digital::Wait,
        spi::{Operation, SpiDevice},
    };

    impl<SPI, ResetPin, InterruptPin, Delay> GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay>
    where
        SPI: SpiDevice,
        ResetPin: OutputPin + Wait,
        InterruptPin: InputPin + Wait,
        Delay: DelayNs,
    {
        pub fn new(
            spi: SPI,
            reset: ResetPin,
            interrupt: InterruptPin,
            delay: Delay,
            slot: u8,
        ) -> GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay> {
            GoModuleUnknown {
                spi,
                reset,
                interrupt,
                delay,
                slot,
            }
        }

        pub async fn module_reset(
            mut self,
        ) -> Result<
            GoModule<SPI, ResetPin, InterruptPin, Delay>,
            GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay>,
        > {
            if self.reset.set_state(PinState::Low).is_err() {
                return Err(self);
            }
            self.delay.delay_ms(100).await;
            if self.reset.set_state(PinState::High).is_err() {
                return Err(self);
            }
            self.delay.delay_ms(100).await;
            Ok(GoModule {
                spi: self.spi,
                reset: self.reset,
                interrupt: self.interrupt,
                delay: self.delay,
                slot: self.slot,
            })
        }
    }

    impl<SPI, ResetPin, InterruptPin, Delay> GoModule<SPI, ResetPin, InterruptPin, Delay>
    where
        SPI: SpiDevice,
        ResetPin: OutputPin + Wait,
        InterruptPin: InputPin + Wait,
        Delay: DelayNs,
    {
        pub async fn escape_module_bootloader(
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
            self.spi
                .transaction(&mut [Operation::Transfer(&mut rx, &tx)])
                .await
                .map_err(GoModuleError::SPI)?;
            Ok(rx)
        }

        pub async fn send_spi(
            &mut self,
            direction: ModuleCommunicationDirection,
            module_id: u8,
            message_type: ModuleCommunicationType,
            message_index: u8,
            tx: &mut [u8],
            delay_us: u16,
        ) -> Result<(), GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
            tx[0] = self.slot;
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
            self.interrupt
                .wait_for_high()
                .await
                .map_err(GoModuleError::InterruptPin)?;

            self.spi
                .transaction(&mut transactions)
                .await
                .map_err(GoModuleError::SPI)?;
            Ok(())
        }

        pub async fn send_receive_spi(
            &mut self,
            direction: ModuleCommunicationDirection,
            module_id: u8,
            message_type: ModuleCommunicationType,
            message_index: u8,
            tx: &mut [u8],
            rx: &mut [u8],
            delay_us: u16,
        ) -> Result<(), GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
            tx[0] = self.slot;
            tx[1] = tx.len() as u8 - 1;
            tx[2] = direction as u8;
            tx[3] = module_id;
            tx[4] = message_type as u8;
            tx[5] = message_index;
            tx[tx.len() - 1] = module_checksum(tx);

            let mut transactions = [
                Operation::DelayNs(delay_us as u32 * 100),
                Operation::Transfer(rx, tx),
            ];
            self.interrupt
                .wait_for_high()
                .await
                .map_err(GoModuleError::InterruptPin)?;
            self.spi
                .transaction(&mut transactions)
                .await
                .map_err(GoModuleError::SPI)?;
            if module_checksum(&rx) == rx[rx.len() - 1] && rx[1] == rx.len() as u8 - 1 {
                Ok(())
            } else {
                Err(GoModuleError::CommunicationError(
                    CommunicationError::ChecksumIncorrect,
                ))
            }
        }

        pub fn get_module_interrupt_state(
            &mut self,
        ) -> Result<PinState, GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>>
        {
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

        pub fn degrade(self) -> GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay> {
            GoModuleUnknown {
                spi: self.spi,
                reset: self.reset,
                interrupt: self.interrupt,
                delay: self.delay,
                slot: self.slot,
            }
        }
    }
}

pub fn module_checksum(data: &[u8], len: usize) -> u8 {
    debug_assert!(len <= data.len());
    let mut checksum: u8 = 0;
    for i in 0..(len - 1) {
        checksum = checksum.wrapping_add(data[i]);
    }
    checksum
}
