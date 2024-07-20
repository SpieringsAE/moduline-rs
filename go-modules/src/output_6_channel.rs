use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
    spi::SpiDevice,
};

use go_module_base::{
    GoModule, GoModuleError, GoModuleUnknown, ModuleCommunicationDirection, ModuleCommunicationType,
};

const OUTPUTMODULE6CHANNELMESSAGELENGHT: usize = 49;
const OUTPUTMODULE6CHANNELID: [u8; 3] = [20, 10, 1];

#[repr(u8)]
#[derive(Clone, Copy, Default)]
pub enum OutputModule6ChannelFunc {
    #[default]
    Disabled = 1,
    HalfBridge,
    LowSideDuty,
    HighSideDuty,
    LowSideBool,
    HighSideBool,
    PeakAndHold(PeakAndHoldSettings),
    Frequency,
}

#[derive(Default)]
pub enum OutputModule6ChannelFrequency {
    #[default]
    Hz1000,
}

#[derive(Clone, Copy, Default)]
pub struct PeakAndHoldSettings {
    pub peak_time: u16,
    pub peak_current: u16,
}

#[derive(Clone, Copy, Default)]
pub struct OutputModule6ChannelChannel {
    func: OutputModule6ChannelFunc,
    max_current: u16,
    peak_and_hold: PeakAndHoldSettings,
}

pub struct OutputModule6ChannelSetpoint {
    pub channel1: u16,
    pub channel2: u16,
    pub channel3: u16,
    pub channel4: u16,
    pub channel5: u16,
    pub channel6: u16,
}

pub struct OutputModule6ChannelValues {
    pub temperature: i16,
    pub ground_shift: u16,
    pub error_code: u32,
    pub supply_volt: u16,
    pub channel1_cur: i16,
    pub channel1_duty: u16,
    pub channel2_cur: i16,
    pub channel2_duty: u16,
    pub channel3_cur: i16,
    pub channel3_duty: u16,
    pub channel4_cur: i16,
    pub channel4_duty: u16,
    pub channel5_cur: i16,
    pub channel5_duty: u16,
    pub channel6_cur: i16,
    pub channel6_duty: u16,
}

#[derive(Default)]
pub struct OutputModule6ChannelConfiguration {
    channels: [OutputModule6ChannelChannel; 6],
    frequencies: [OutputModule6ChannelFrequency; 3],
}

pub struct OutputModule6Channel<SPI, ResetPin, InterruptPin, Delay> {
    module: GoModule<SPI, ResetPin, InterruptPin, Delay>,
    configuration: OutputModule6ChannelConfiguration,
}

pub struct OutputModule6ChannelBuilder<SPI, ResetPin, InterruptPin, Delay> {
    module: GoModule<SPI, ResetPin, InterruptPin, Delay>,
    configuration: OutputModule6ChannelConfiguration,
}

impl<SPI, ResetPin, InterruptPin, Delay> OutputModule6Channel<SPI, ResetPin, InterruptPin, Delay>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
    Delay: DelayNs,
{
}
