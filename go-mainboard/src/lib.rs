pub enum ControllerType {
    #[cfg(feature = "std")]
    ModulineIV(HwVersion),
    #[cfg(feature = "std")]
    ModulineIII(HwVersion),
    #[cfg(not(feature = "std"))]
    ModulineII(HwVersion),
    #[cfg(feature = "std")]
    ModulineMini(HwVersion),
    #[cfg(feature = "std")]
    ModulineDisplay(HwVersion),
}

pub struct HwVersion {
    version_major: u8,
    version_minor: u8,
}

#[repr(u8)]
pub enum ControllerSlot {
    Slot1,
    Slot2,
    Slot3,
    Slot4,
    Slot5,
    Slot6,
    Slot7,
    Slot8,
}

pub fn get_controller_type() -> Result<ControllerType, ()> {
    #[cfg(not(feature = "std"))]
    {
        Ok(ControllerType::ModulineII(HwVersion {
            //default to V1.04
            version_major: 1,
            version_minor: 4,
        }))
    }
    #[cfg(feature = "std")]
    {
        let hw =
            std::fs::read_to_string("/sys/firmware/devicetree/base/hardware").map_err(|err| {
                eprintln!("{:?}", err);
                ()
            })?;
        let mut version = hw
            .split(" ")
            .last()
            .ok_or_else(|| eprintln!("failed to get version part of hardware"))?
            .split(".");
        let version_major = u8::from_str_radix(
            version
                .nth(0)
                .ok_or_else(|| eprintln!("failed to get hw version major"))?,
            10,
        )
        .map_err(|err| {
            eprintln!("failed to parse controller hw version major\n{:?}", err);
            ()
        })?;
        let version_minor = u8::from_str_radix(
            version
                .nth(1)
                .ok_or_else(|| eprintln!("failed to get hw version minor"))?,
            10,
        )
        .map_err(|err| {
            eprintln!("failed to parse controller hw version minor\n{:?}", err);
            ()
        })?;
        if hw.contains("Moduline IV") {
            Ok(ControllerType::ModulineIV(HwVersion {
                version_major,
                version_minor,
            }))
        } else if hw.contains("Moduline Mini") {
            Ok(ControllerType::ModulineMini(HwVersion {
                version_major,
                version_minor,
            }))
        } else if hw.contains("Moduline Display") {
            Ok(ControllerType::ModulineDisplay(HwVersion {
                version_major,
                version_minor,
            }))
        } else if hw.contains("Moduline III") {
            Ok(ControllerType::ModulineIII(HwVersion {
                version_major,
                version_minor,
            }))
        } else {
            Err(())
        }
    }
}

impl<SPI, ResetPin, InterruptPin> GoModule<SPI, ResetPin, InterruptPin>
where
    SPI: SpiDevice,
    ResetPin: OutputPin,
    InterruptPin: InputPin,
{
    #[cfg(feature = "std")]
    ///Get all of the available modules for the given controller type
    ///These modules can then be embedded into function specific modules like the InputModule6Channel
    pub fn get_modules(
        controller_type: ControllerType,
        slot: ControllerSlot,
    ) -> Result<GoModules<SpidevDevice, CdevPin, CdevPin>, ModuleSetupError> {
        extern crate std;
        match controller_type {
            ControllerType::ModulineIV(_) => {
                return Ok(GoModules([
                    Some(GoModule {
                        spi: get_module_spi("/dev/spidev1.0")?,
                        reset: get_module_reset("gpiochip0", 7, ControllerSlot::Slot1)?,
                        interrupt: get_module_interrupt("gpiochip0", 6, ControllerSlot::Slot1)?,
                        slot: ControllerSlot::Slot1,
                    }),
                    Some(GoModule {
                        spi: get_module_spi("/dev/spidev1.1")?,
                        reset: get_module_reset("gpiochip2", 10, ControllerSlot::Slot2)?,
                        interrupt: get_module_interrupt("gpiochip4", 20, ControllerSlot::Slot2)?,
                        slot: ControllerSlot::Slot2
                    }),
                    Some(GoModule {
                        spi: get_module_spi("/dev/spidev2.0")?,
                        reset: get_module_reset("gpiochip0", 4, ControllerSlot::Slot3)?,
                        interrupt: get_module_interrupt("gpiochip0", 7, ControllerSlot::Slot3)?,
                        slot: ControllerSlot::Slot3,
                    }),
                    Some(GoModule {
                        spi: get_module_spi("/dev/spidev2.1")?,
                        reset: get_module_reset("gpiochip0", 2, ControllerSlot::Slot4)?,
                        interrupt: get_module_interrupt("gpiochip4", 21, ControllerSlot::Slot4)?,
                        slot: ControllerSlot::Slot4,
                    }),
                    Some(GoModule {
                        spi: get_module_spi("/dev/spidev2.2")?,
                        reset: get_module_reset("gpiochip3", 24, ControllerSlot::Slot5)?,
                        interrupt: get_module_interrupt("gpiochip4", 1, ControllerSlot::Slot5)?,
                        slot: ControllerSlot::Slot5,
                    }),
                    Some(GoModule {
                        spi: get_module_spi("/dev/spidev2.3")?,
                        reset: get_module_reset("gpiochip3", 27, ControllerSlot::Slot6)?,
                        interrupt: get_module_interrupt("gpiochip3", 26, ControllerSlot::Slot6)?,
                        slot: ControllerSlot::Slot6,
                    }),
                    Some(GoModule {
                        spi: get_module_spi("/dev/spidev0.0")?,
                        reset: get_module_reset("gpiochip2", 24, ControllerSlot::Slot7)?,
                        interrupt: get_module_interrupt("gpiochip2", 19, ControllerSlot::Slot7)?,
                        slot: ControllerSlot::Slot7,
                    }),
                    Some(GoModule {
                        spi: get_module_spi("/dev/spidev0.1")?,
                        reset: get_module_reset("gpiochip2", 20, ControllerSlot::Slot8)?,
                        interrupt: get_module_interrupt("gpiochip2", 22, ControllerSlot::Slot8)?,
                        slot: ControllerSlot::Slot8,
                    }),
                ]))
            },
            _ => todo!("implement all the hardware"),
        };
    }

    #[cfg(not(feature = "std"))]
    pub fn new(controller_type: ControllerType, slot: ControllerSlot, dp: &mut Peripherals) -> Result<Self, ModuleSetupError> {
        let (interrupt, reset, spi) = match ControllerType {
            ControllerType::ModulineII(_) => {
                match slot {
                    ControllerSlot::Slot1 => {
                        todo!("figure out Moduline 2 stuff")
                    }
                    _=> todo!("more slots")
                }
            }
        };
        Ok(GoModule { spi, reset, interrupt })
    }
}

#[cfg(feature = "std")]
fn get_module_interrupt(
    chip: &str,
    line: u32,
    slot: ControllerSlot,
) -> Result<CdevPin, ModuleSetupError> {
    extern crate std;
    let mut chip = Chip::new(chip).map_err(|_| ModuleSetupError::InterruptPin)?;
    let line = chip
        .get_line(line)
        .map_err(|_| ModuleSetupError::InterruptPin)?;
    let line_handle = line
        .request(
            LineRequestFlags::INPUT,
            0,
            std::format!("slot {} module interrupt", slot as u8 + 1).as_str(),
        )
        .map_err(|_| ModuleSetupError::InterruptPin)?;
    Ok(CdevPin::new(line_handle).map_err(|_| ModuleSetupError::InterruptPin)?)
}

#[cfg(feature = "std")]
fn get_module_reset(
    chip: &str,
    line: u32,
    slot: ControllerSlot,
) -> Result<CdevPin, ModuleSetupError> {
    extern crate std;
    let mut chip = Chip::new(chip).map_err(|_| ModuleSetupError::InterruptPin)?;
    let line = chip.get_line(line).map_err(|_| ModuleSetupError::ResetPin)?;
    let line_handle = line
        .request(
            LineRequestFlags::OUTPUT,
            0,
            std::format!("slot {} module reset", slot as u8 + 1).as_str(),
        )
        .map_err(|_| ModuleSetupError::ResetPin)?;
    Ok(CdevPin::new(line_handle).map_err(|_| ModuleSetupError::ResetPin)?)
}

#[cfg(feature = "std")]
fn get_module_spi(spidev: &str) -> Result<SpidevDevice, ModuleSetupError> {
    extern crate std;
    let mut spi = SpidevDevice::open(spidev).map_err(|_| ModuleSetupError::Spi)?;
    let spi_opts = SpidevOptions {
        bits_per_word: Some(8),
        max_speed_hz: Some(2_000_000),
        lsb_first: None,
        spi_mode: Some(SpiModeFlags::SPI_MODE_0),
    };
    spi.configure(&spi_opts)
        .map_err(|_| ModuleSetupError::Spi)?;
    Ok(spi)
}
