use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
    spi::SpiDevice,
};
use go_module_base::{
    GoModule, GoModuleError, GoModuleUnknown, ModuleCommunicationDirection, ModuleCommunicationType,
};

const INPUTMODULE6CHANNELMESSAGELENGTH: usize = 55;
const INPUTMODULE6CHANNELID: [u8; 3] = [20, 10, 1];
const RESISTORMATRIX: [u8; 4] = [0, 3, 1, 2];

#[repr(u8)]
#[derive(Clone, Copy)]
/// Selects the funcion of a given input channel, some functions require two channels
pub enum InputModule6ChannelFunc {
    /// Get the raw adc value, u16 is the number of samples
    AnalogRaw(u16) = 1,
    /// Get the voltage in mV, u16 is the number of samples
    AnalogmV(u16),
    /// Read a digital signal, high or low
    Digital,
    /// Measure the frequency of a signal
    Frequency,
    /// Measure the low period of a duty cycle signal
    DutyLow,
    /// Measure the high period of a duty cycle signal
    DutyHigh,
    /// RPM mode, u8 is the number of pulses per rotation
    /// Combined with the adjacent channel 1&2, 3&4, 5&6
    RPM(u8),
    /// Count pulses on this channel
    /// Combined with the adjacent channel 1&2, 3&4, 5&6
    PulseCounter,
}

impl InputModule6ChannelFunc {
    fn discriminant(&self) -> u8 {
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
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
    PD3_3k,
    PD4_7k,
    PD10k,
}

#[repr(u8)]
#[derive(Clone, Copy)]
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
    pub channel1: u32,
    pub channel2: u32,
    pub channel3: u32,
    pub channel4: u32,
    pub channel5: u32,
    pub channel6: u32,
}

#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub enum InputModule6ChannelSupply {
    #[default]
    On = 1,
    Off,
}

#[derive(Default)]
pub struct InputModule6ChannelConfiguration {
    channels: [InputModule6ChannelChannel; 6],
    supplies: [InputModule6ChannelSupply; 3],
}

pub struct InputModule6Channel<SPI, ResetPin, InterruptPin, Delay> {
    module: GoModule<SPI, ResetPin, InterruptPin, Delay>,
    configuration: InputModule6ChannelConfiguration,
}

impl<SPI, ResetPin, InterruptPin, Delay> InputModule6Channel<SPI, ResetPin, InterruptPin, Delay>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
    Delay: DelayNs,
{
    pub fn reconfigure(
        self,
    ) -> (
        GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay>,
        InputModule6ChannelConfiguration,
    ) {
        (self.module.degrade(), self.configuration)
    }

    pub fn read_channels(
        &mut self,
    ) -> Result<
        InputModule6ChannelValues,
        GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>,
    > {
        let mut tx = [0u8; INPUTMODULE6CHANNELMESSAGELENGTH + 5];
        let mut rx = [0u8; INPUTMODULE6CHANNELMESSAGELENGTH + 5];
        self.module.send_receive_spi(
            ModuleCommunicationDirection::ToModule,
            11,
            ModuleCommunicationType::Data,
            1,
            &mut tx,
            &mut rx,
            INPUTMODULE6CHANNELMESSAGELENGTH,
            0,
        )?;
        if rx[2] != ModuleCommunicationDirection::FromModule as u8
            || rx[3] != 11
            || rx[4] != ModuleCommunicationType::Data as u8
            || rx[5] != 1
        {
            return Err(GoModuleError::CommunicationError(
                go_module_base::CommunicationError::UnableToSerDe,
            ));
        }
        Ok(InputModule6ChannelValues {
            channel1: u32::from_le_bytes(rx[6..10].try_into().unwrap()), //These can't fail aslong as the slice is correctly sized
            channel2: u32::from_le_bytes(rx[14..18].try_into().unwrap()),
            channel3: u32::from_le_bytes(rx[22..26].try_into().unwrap()),
            channel4: u32::from_le_bytes(rx[30..34].try_into().unwrap()),
            channel5: u32::from_le_bytes(rx[38..42].try_into().unwrap()),
            channel6: u32::from_le_bytes(rx[46..50].try_into().unwrap()),
        })
    }

    pub fn reset_counter(
        &mut self,
        channel: InputModule6ChannelNum,
        value: i32,
    ) -> Result<(), GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>> {
        let mut tx = [0u8; INPUTMODULE6CHANNELMESSAGELENGTH + 5];
        tx[6] = channel as u8;
        tx[7..11].copy_from_slice(&value.to_le_bytes());
        self.module.send_spi(
            ModuleCommunicationDirection::ToModule,
            11,
            ModuleCommunicationType::Data,
            2,
            &mut tx,
            INPUTMODULE6CHANNELMESSAGELENGTH,
            0,
        )
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

impl InputModule6ChannelConfiguration {
    fn serialize(&self, tx: &mut [u8]) {
        for (i, channel) in self.channels.iter().enumerate() {
            let samples = match channel.func {
                InputModule6ChannelFunc::AnalogRaw(samples) => samples.clamp(0, 1000),
                InputModule6ChannelFunc::AnalogmV(samples) => samples.clamp(0, 1000),
                InputModule6ChannelFunc::RPM(pulses) => (pulses.clamp(0, 200) as u16) << 8,
                _ => 0,
            };
            tx[6 + i * 6] = channel.func.discriminant();
            tx[7 + i * 6] = RESISTORMATRIX[channel.pu as usize]
                | RESISTORMATRIX[channel.pd as usize] << 2
                | (channel.volt as u8) << 6;
            tx[8 + i * 6] = (samples >> 8) as u8;
            tx[9 + i * 6] = samples as u8;
            tx[10 + i * 6] = 0;
            tx[11 + i * 6] = 0;
            tx[42 + i / 2] = self.supplies[i / 2] as u8;
        }
    }
}

pub struct InputModule6ChannelBuilder<SPI, ResetPin, InterruptPin, Delay> {
    module: GoModule<SPI, ResetPin, InterruptPin, Delay>,
    config: InputModule6ChannelConfiguration,
}

impl<SPI, ResetPin, InterruptPin, Delay>
    InputModule6ChannelBuilder<SPI, ResetPin, InterruptPin, Delay>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
    Delay: DelayNs,
{
    pub fn new(module: GoModule<SPI, ResetPin, InterruptPin, Delay>) -> Self {
        InputModule6ChannelBuilder {
            module,
            config: InputModule6ChannelConfiguration::default(),
        }
    }

    pub fn from_configuration(
        module: GoModule<SPI, ResetPin, InterruptPin, Delay>,
        config: InputModule6ChannelConfiguration,
    ) -> Self {
        InputModule6ChannelBuilder { module, config }
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
        config.channels[channel as usize - 1] = InputModule6ChannelChannel { func, pu, pd, volt };
        InputModule6ChannelBuilder {
            module: self.module,
            config,
        }
    }

    pub fn configure_supplies(
        self,
        supply1: InputModule6ChannelSupply,
        supply2: InputModule6ChannelSupply,
        supply3: InputModule6ChannelSupply,
    ) -> Self {
        let mut config = self.config;
        config.supplies[0] = supply1;
        config.supplies[1] = supply2;
        config.supplies[2] = supply3;
        InputModule6ChannelBuilder {
            module: self.module,
            config,
        }
    }

    pub fn build(
        self,
    ) -> Result<
        InputModule6Channel<SPI, ResetPin, InterruptPin, Delay>,
        (
            GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay>,
            InputModule6ChannelConfiguration,
        ),
    > {
        let mut module = InputModule6Channel {
            module: self.module,
            configuration: self.config,
        };
        let Ok(bootmessage) = module.module.escape_module_bootloader() else {
            return Err((module.module.degrade(), module.configuration));
        };

        if INPUTMODULE6CHANNELID != bootmessage[6..9] {
            return Err((module.module.degrade(), module.configuration));
        }
        let mut tx = [0u8; INPUTMODULE6CHANNELMESSAGELENGTH + 5];
        module.configuration.serialize(&mut tx);
        if module
            .module
            .send_spi(
                ModuleCommunicationDirection::ToModule,
                11,
                ModuleCommunicationType::Configuration,
                1,
                &mut tx,
                INPUTMODULE6CHANNELMESSAGELENGTH,
                500_000, //for some reason it takes very long to exit the bootloader?
            )
            .is_err()
        {
            return Err((module.module.degrade(), module.configuration));
        }
        Ok(module)
    }
}
