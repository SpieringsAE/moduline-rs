use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
    spi::SpiDevice,
};

use go_module_base::{
    GoModule, GoModuleError, GoModuleUnknown, ModuleCommunicationDirection, ModuleCommunicationType,
};

const OUTPUTMODULE6CHANNELMESSAGELENGTH: usize = 44;
const OUTPUTMODULE6CHANNELID: [u8; 3] = [20, 20, 2];

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

impl OutputModule6ChannelFunc {
    fn discriminant(&self) -> u8 {
        //This is only safe to do on enums with #[repr(u8)].
        //Sadly there seems to be no inherently safe method to do this.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub enum OutputModule6ChannelFrequency {
    Hz100 = 1,
    Hz200,
    Hz500,
    #[default]
    Hz1_000,
    Hz2_000,
    Hz5_000,
    Hz10_000,
}

#[repr(usize)]
pub enum OutputModule6ChannelNum {
    One = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
}

#[repr(usize)]
pub enum OutputModule6ChannelFrequencyNum {
    OneTwo,
    ThreeFour,
    FiveSix,
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

impl OutputModule6ChannelSetpoint {
    fn serialize(&self, tx: &mut [u8]) {
        tx[6..8].copy_from_slice(&self.channel1.to_le_bytes());
        tx[12..14].copy_from_slice(&self.channel2.to_le_bytes());
        tx[18..20].copy_from_slice(&self.channel3.to_le_bytes());
        tx[24..26].copy_from_slice(&self.channel4.to_le_bytes());
        tx[30..32].copy_from_slice(&self.channel5.to_le_bytes());
        tx[36..38].copy_from_slice(&self.channel6.to_le_bytes());
    }
}

impl OutputModule6ChannelConfiguration {
    fn serialize1(&self, tx: &mut [u8]) {
        for (i, channel) in self.channels.iter().enumerate() {
            let func_byte = channel.func.discriminant() << 4 | self.frequencies[i / 2] as u8;
            tx[6 + i] = func_byte;
            tx[12 + i * 2..14 + i * 2].copy_from_slice(&channel.max_current.to_le_bytes())
        }
    }

    fn serialize2(&self, tx: &mut [u8]) {
        for (i, channel) in self.channels.iter().enumerate() {
            match channel.func {
                OutputModule6ChannelFunc::PeakAndHold(settings) => {
                    tx[6 + i * 2..8 + i * 2].copy_from_slice(&settings.peak_current.to_le_bytes());
                    tx[18 + i * 2..20 + i * 2].copy_from_slice(&settings.peak_time.to_le_bytes());
                }
                _ => {}
            }
        }
    }
}

impl<SPI, ResetPin, InterruptPin, Delay> OutputModule6Channel<SPI, ResetPin, InterruptPin, Delay>
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
        OutputModule6ChannelConfiguration,
    ) {
        (self.module.degrade(), self.configuration)
    }

    pub fn set_and_read_channels(
        &mut self,
        setpoint: &OutputModule6ChannelSetpoint,
    ) -> Result<
        OutputModule6ChannelValues,
        GoModuleError<SPI::Error, ResetPin::Error, InterruptPin::Error>,
    > {
        let mut tx = [0u8; OUTPUTMODULE6CHANNELMESSAGELENGTH + 5];
        let mut rx = [0u8; OUTPUTMODULE6CHANNELMESSAGELENGTH + 5];
        setpoint.serialize(&mut tx);
        self.module.send_receive_spi(
            ModuleCommunicationDirection::ToModule,
            22,
            ModuleCommunicationType::Data,
            1,
            &mut tx,
            &mut rx,
            OUTPUTMODULE6CHANNELMESSAGELENGTH,
            0,
        )?;
        if rx[2] != ModuleCommunicationDirection::FromModule as u8
            || rx[3] != 22
            || rx[4] != ModuleCommunicationType::Feedback as u8
            || rx[5] != 1
        {
            return Err(GoModuleError::CommunicationError(
                go_module_base::CommunicationError::UnableToSerDe,
            ));
        }
        Ok(OutputModule6ChannelValues {
            temperature: i16::from_le_bytes(rx[6..8].try_into().unwrap()),
            ground_shift: u16::from_le_bytes(rx[8..10].try_into().unwrap()),
            channel1_cur: i16::from_le_bytes(rx[10..12].try_into().unwrap()),
            channel2_cur: i16::from_le_bytes(rx[12..14].try_into().unwrap()),
            channel3_cur: i16::from_le_bytes(rx[14..16].try_into().unwrap()),
            channel4_cur: i16::from_le_bytes(rx[16..18].try_into().unwrap()),
            channel5_cur: i16::from_le_bytes(rx[18..20].try_into().unwrap()),
            channel6_cur: i16::from_le_bytes(rx[20..22].try_into().unwrap()),
            error_code: u32::from_le_bytes(rx[22..26].try_into().unwrap()),
            channel1_duty: u16::from_le_bytes(rx[26..28].try_into().unwrap()),
            channel2_duty: u16::from_le_bytes(rx[28..30].try_into().unwrap()),
            channel3_duty: u16::from_le_bytes(rx[30..32].try_into().unwrap()),
            channel4_duty: u16::from_le_bytes(rx[32..34].try_into().unwrap()),
            channel5_duty: u16::from_le_bytes(rx[34..36].try_into().unwrap()),
            channel6_duty: u16::from_le_bytes(rx[36..38].try_into().unwrap()),
            supply_volt: u16::from_le_bytes(rx[41..43].try_into().unwrap()),
        })
    }
}

impl<SPI, ResetPin, InterruptPin, Delay>
    OutputModule6ChannelBuilder<SPI, ResetPin, InterruptPin, Delay>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
    Delay: DelayNs,
{
    pub fn new(module: GoModule<SPI, ResetPin, InterruptPin, Delay>) -> Self {
        OutputModule6ChannelBuilder {
            module,
            configuration: OutputModule6ChannelConfiguration::default(),
        }
    }

    pub fn from_configuration(
        module: GoModule<SPI, ResetPin, InterruptPin, Delay>,
        configuration: OutputModule6ChannelConfiguration,
    ) -> Self {
        OutputModule6ChannelBuilder {
            module,
            configuration,
        }
    }

    pub fn configure_channel(
        self,
        channel: OutputModule6ChannelNum,
        func: OutputModule6ChannelFunc,
        max_current: u16,
    ) -> Self {
        let mut configuration = self.configuration;
        configuration.channels[channel as usize - 1] =
            OutputModule6ChannelChannel { func, max_current };
        OutputModule6ChannelBuilder {
            module: self.module,
            configuration,
        }
    }

    pub fn configure_frequency(
        self,
        channel: OutputModule6ChannelFrequencyNum,
        freq: OutputModule6ChannelFrequency,
    ) -> Self {
        let mut configuration = self.configuration;
        configuration.frequencies[channel as usize] = freq;
        OutputModule6ChannelBuilder {
            module: self.module,
            configuration,
        }
    }

    pub fn build(
        self,
    ) -> Result<
        OutputModule6Channel<SPI, ResetPin, InterruptPin, Delay>,
        (
            GoModuleUnknown<SPI, ResetPin, InterruptPin, Delay>,
            OutputModule6ChannelConfiguration,
        ),
    > {
        let mut module = OutputModule6Channel {
            module: self.module,
            configuration: self.configuration,
        };
        let Ok(bootmessage) = module.module.escape_module_bootloader() else {
            return Err((module.module.degrade(), module.configuration));
        };

        if OUTPUTMODULE6CHANNELID != bootmessage[6..9] {
            return Err((module.module.degrade(), module.configuration));
        }

        let mut tx = [0u8; OUTPUTMODULE6CHANNELMESSAGELENGTH + 5];
        module.configuration.serialize1(&mut tx);
        if module
            .module
            .send_spi(
                ModuleCommunicationDirection::ToModule,
                22,
                ModuleCommunicationType::Configuration,
                1,
                &mut tx,
                OUTPUTMODULE6CHANNELMESSAGELENGTH,
                500_000,
            )
            .is_err()
        {
            return Err((module.module.degrade(), module.configuration));
        }
        module.configuration.serialize2(&mut tx);
        if module
            .module
            .send_spi(
                ModuleCommunicationDirection::ToModule,
                22,
                ModuleCommunicationType::Configuration,
                2,
                &mut tx,
                OUTPUTMODULE6CHANNELMESSAGELENGTH,
                500,
            )
            .is_err()
        {
            return Err((module.module.degrade(), module.configuration));
        }
        Ok(module)
    }
}
