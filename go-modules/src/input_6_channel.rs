use std::io::{Cursor, Write};

use embedded_hal::{
    digital::{InputPin, OutputPin},
    spi::SpiDevice,
};
use go_module_base::{GoModule, GoModuleError, ModuleCommunicationDirection, ModuleCommunicationType};


const INPUTMODULE6CHANNELMESSAGELENGTH: usize = 55;
const INPUTMODULE6CHANNELID: [u8; 3] = [20, 10, 1];
const RESISTORMATRIX: [u8;4] = [0,3,1,2];

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum InputModule6ChannelFunc {
    AnalogRaw(u16) = 1,
    AnalogmV(u16),
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

#[derive(Clone,Copy)]
pub enum InputModule6ChannelVoltage {
	Voltage5V,
	Voltage12V,
	Voltage24V,
}

#[repr(usize)]
pub enum InputModule6ChannelNum {
	One = 1,
	Two,
	Three,
	Four,
	Five,
	Six,
}

pub struct InputModule6ChannelValues {
	pub channel1:u32,
	pub channel2:u32,
	pub channel3:u32,
	pub channel4:u32,
	pub channel5:u32,
	pub channel6:u32,
}

#[derive(Default, Clone, Copy)]
pub enum InputModule6ChannelSupply {
	Off,
	#[default] On,
}

#[derive(Default)]
pub struct InputModule6ChannelConfiguration {
	channels: [InputModule6ChannelChannel;6],
	supplies: [InputModule6ChannelSupply;3],
}

pub struct InputModule6Channel<SPI, ResetPin, InterruptPin> {
    module: GoModule<SPI, ResetPin, InterruptPin>,
    configuration: InputModule6ChannelConfiguration,
}

impl<SPI, ResetPin, InterruptPin> InputModule6Channel<SPI, ResetPin, InterruptPin>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
{
    pub fn reconfigure(
        self,
    ) -> (GoModule<SPI, ResetPin, InterruptPin>,InputModule6ChannelConfiguration)
     {
		(self.module, self.configuration)
    }

	pub fn read_channels(&mut self) -> Result<InputModule6ChannelValues, GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
		let mut tx = [0u8;INPUTMODULE6CHANNELMESSAGELENGTH];
		let mut rx = [0u8;INPUTMODULE6CHANNELMESSAGELENGTH];
		self.module.send_receive_spi(ModuleCommunicationDirection::FromModule, 11, ModuleCommunicationType::Data, 1, &mut tx, &mut rx, 0)?;
		Ok(InputModule6ChannelValues {
			channel1: u32::from_le_bytes(rx[6..10].try_into().unwrap()), //These can't fail aslong as the slice is correctly sized
			channel2: u32::from_le_bytes(rx[14..18].try_into().unwrap()),
			channel3: u32::from_le_bytes(rx[22..26].try_into().unwrap()),
			channel4: u32::from_le_bytes(rx[30..34].try_into().unwrap()),
			channel5: u32::from_le_bytes(rx[38..42].try_into().unwrap()),
			channel6: u32::from_le_bytes(rx[46..50].try_into().unwrap()),
		})
	}

	pub fn reset_counter(&mut self, channel: InputModule6ChannelNum, value: i32) -> Result<(), GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
		let tx = [0u8;INPUTMODULE6CHANNELMESSAGELENGTH];
		let mut cursor = Cursor::new(tx);
		cursor.set_position(6);
		cursor.write(&[channel as u8]).unwrap();
		cursor.write(&value.to_le_bytes()).unwrap();
		self.module.send_spi(ModuleCommunicationDirection::ToModule, 11, ModuleCommunicationType::Data, 2, cursor.get_mut(), 0)
	}
}

#[derive(Clone, Copy)]
pub struct InputModule6ChannelChannel {
    func: InputModule6ChannelFunc,
    pu: InputModule6ChannelPullUp,
    pd: InputModule6ChannelPullDown,
	volt: InputModule6ChannelVoltage,
}

impl Default for InputModule6ChannelChannel {
    fn default() -> Self {
        InputModule6ChannelChannel {
            func: InputModule6ChannelFunc::AnalogmV(1000),
            pu: InputModule6ChannelPullUp::None,
            pd: InputModule6ChannelPullDown::None,
			volt: InputModule6ChannelVoltage::Voltage5V,
        }
    }
}

impl InputModule6ChannelChannel {
	fn serialize(&self) -> [u8;6] {
		let (func,samples) = match self.func {
			InputModule6ChannelFunc::AnalogRaw(samples) => (1,samples),
			InputModule6ChannelFunc::AnalogmV(samples) => (2,samples),
		};
		[
			func,
			RESISTORMATRIX[self.pu as usize] | RESISTORMATRIX[self.pd as usize] << 2 | (self.volt as u8) << 6,
			(samples  >> 8) as u8,
			samples as u8,
			0,
			0,
		]
	}
}

pub struct InputModule6ChannelBuilder<SPI, ResetPin, InterruptPin> {
    module: GoModule<SPI, ResetPin, InterruptPin>,
	config: InputModule6ChannelConfiguration,
}

impl<SPI, ResetPin, InterruptPin> InputModule6ChannelBuilder<SPI, ResetPin, InterruptPin>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
{
    pub fn new(module: GoModule<SPI, ResetPin, InterruptPin>) -> Self {
        InputModule6ChannelBuilder {
            module,
			config: InputModule6ChannelConfiguration::default(),
        }
    }

    pub fn configure_channel(
        self,
		channel: InputModule6ChannelNum,
        func: InputModule6ChannelFunc,
        pu: InputModule6ChannelPullUp,
        pd: InputModule6ChannelPullDown,
		volt: InputModule6ChannelVoltage,
    ) -> Self {
		let mut config = self.config;
		config.channels[channel as usize] = InputModule6ChannelChannel{ func, pu, pd, volt};
        InputModule6ChannelBuilder {
            module: self.module,
            config: config,
        }
    }

    pub fn build(self) -> Result<InputModule6Channel<SPI, ResetPin, InterruptPin>, (GoModule<SPI,ResetPin,InterruptPin>, [InputModule6ChannelChannel;6])> {
        let module = InputModule6Channel {
            module: self.module,
            configuration: self.config
        };
		let tx = [0u8;INPUTMODULE6CHANNELMESSAGELENGTH];
		let mut cursor = Cursor::new(tx);
		cursor.set_position(6);
		for channel in &module.configuration.channels {
			cursor.write(&channel.serialize()).unwrap();
		}
		for supply in &module.configuration.supplies {
			cursor.write(&[*supply as u8]).unwrap();
		}
		Ok(module)
    }
}
