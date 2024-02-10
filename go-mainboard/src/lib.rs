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
