use crate::go_module::{GoModule, GoModuleError};
use embedded_hal::{
    digital::{InputPin, OutputPin},
    spi::SpiDevice,
};

const INPUTMODULE6CHANNELMESSAGELENGTH: usize = 55;
const INPUTMODULE6CHANNELID: [u8; 3] = [20, 10, 1];

pub struct InputModule6Channel<SPI, ResetPin, InterruptPin> {
    module: GoModule<SPI, ResetPin, InterruptPin>,
}

impl<SPI, ResetPin, InterruptPin> InputModule6Channel<SPI, ResetPin, InterruptPin>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
{
    pub fn new(
        module: GoModule<SPI, ResetPin, InterruptPin>,
    ) -> Result<Self, GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
        Ok(InputModule6Channel { module })
    }
}
