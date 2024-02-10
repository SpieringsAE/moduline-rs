use crate::go_module::{GoModule, GoModuleError};
use embedded_hal::{
    digital::{InputPin, OutputPin},
    spi::SpiDevice,
};
use go_mainboard::{ControllerSlot, ControllerType};

const INPUTMODULE6CHANNELMESSAGELENGTH: usize = 55;
const INPUTMODULE6CHANNELID: [u8; 3] = [20, 10, 1];

pub struct InputModule6Channel<SPI, ResetPin, InterruptPin> {
    module: GoModule<SPI, ResetPin, InterruptPin>,
    module_slot: ControllerSlot,
}

impl<SPI, ResetPin, InterruptPin> InputModule6Channel<SPI, ResetPin, InterruptPin>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
{
    pub fn new(
        controller: ControllerType,
        slot: ControllerSlot,
    ) -> Result<Self, GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
        let module = GoModule::<SPI, ResetPin, InterruptPin>::new(controller, slot)
            .map_err(GoModuleError::ModuleSetupError)?;
        Ok(InputModule6Channel {
            module,
            module_slot: slot,
        })
    }
}
