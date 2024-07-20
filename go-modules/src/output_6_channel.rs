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
#[derive(Clone, Copy)]
pub enum OutputModule6ChannelFunc {
    Disabled = 1,
    HalfBridge,
    LowSideDuty,
    HighSideDuty,
    LowSideBool,
    HighSideBool,
    PeakAndHold(PeakAndHoldSettings),
    Frequency,
}

#[derive(Clone, Copy, Default)]
pub struct PeakAndHoldSettings {
    pub peak_time: u16,
    pub peak_current: u16,
}

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
    pub temperature
