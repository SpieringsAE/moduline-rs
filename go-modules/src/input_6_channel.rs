use embedded_hal::{
    digital::{InputPin, OutputPin},
    spi::SpiDevice,
};
use go_module_base::{GoModule, GoModuleError};

const INPUTMODULE6CHANNELMESSAGELENGTH: usize = 55;
const INPUTMODULE6CHANNELID: [u8; 3] = [20, 10, 1];

#[derive(Clone, Copy)]
pub enum InputModule6ChannelFunc {
    AnalogRaw,
    AnalogmV,
}

#[derive(Clone, Copy)]
pub enum InputModule6ChannelPullUp {
    None,
    PU3_3k,
    PU4_7k,
    PU10k,
}

#[derive(Clone, Copy)]
pub enum InputModule6ChannelPullDown {
    None,
    PU3_3k,
    PU4_7k,
    PU10k,
}

#[repr(usize)]
pub enum InputModule6ChannelNum {
	One,
	Two,
	Three,
	Four,
	Five,
	Six,
}

pub struct InputModule6Channel<SPI, ResetPin, InterruptPin> {
    module: GoModule<SPI, ResetPin, InterruptPin>,
    channels: [InputModule6ChannelChannel;6],
}

impl<SPI, ResetPin, InterruptPin> InputModule6Channel<SPI, ResetPin, InterruptPin>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
{
    pub fn reconfigure(
        self,
    ) -> InputModule6ChannelBuilder<SPI, ResetPin, InterruptPin>
     {
        InputModule6ChannelBuilder {
			module: self.module,
            channels: self.channels,
		}
    }
}

#[derive(Clone, Copy)]
struct InputModule6ChannelChannel {
    func: InputModule6ChannelFunc,
    pu: InputModule6ChannelPullUp,
    pd: InputModule6ChannelPullDown,
}

impl Default for InputModule6ChannelChannel {
    fn default() -> Self {
        InputModule6ChannelChannel {
            func: InputModule6ChannelFunc::AnalogRaw,
            pu: InputModule6ChannelPullUp::None,
            pd: InputModule6ChannelPullDown::None,
        }
    }
}

pub struct InputModule6ChannelBuilder<SPI, ResetPin, InterruptPin> {
    module: GoModule<SPI, ResetPin, InterruptPin>,
	channels: [InputModule6ChannelChannel;6],

}

impl<SPI, ResetPin, InterruptPin> InputModule6ChannelBuilder<SPI, ResetPin, InterruptPin>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
{
    pub fn new(spi: SPI, reset: ResetPin, int: InterruptPin) -> Self {
        InputModule6ChannelBuilder {
            module: GoModule::new(spi, reset, int),
			channels: [InputModule6ChannelChannel::default();6],
        }
    }

    pub fn configure_channel(
        self,
		channel: InputModule6ChannelNum,
        func: InputModule6ChannelFunc,
        pu: InputModule6ChannelPullUp,
        pd: InputModule6ChannelPullDown,
    ) -> Self {
		let mut channels = self.channels;
		channels[channel as usize] = InputModule6ChannelChannel{ func, pu, pd};
        InputModule6ChannelBuilder {
            module: self.module,
            channels: channels,
        }
    }

    pub fn build(self) -> InputModule6Channel<SPI, ResetPin, InterruptPin> {
        InputModule6Channel {
            module: self.module,
            channels: self.channels
        }
    }
}
