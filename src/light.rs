use ddc_hi::{Ddc, Display};
pub fn get_light() -> u16 {
    // for mut display in Display::enumerate() {
    //     display.update_capabilities().unwrap();
    //     if let Some(feature) = display.info.mccs_database.get(0x10) {
    //         if let Ok(value) = display.handle.get_vcp_feature(feature.code) {
    //             return value.value();
    //         }
    //     }
    // }
    0
}

pub fn set_light(v: u16) {
    for (i, mut display) in Display::enumerate().into_iter().enumerate() {
        println!("i: {i}");
        display.update_capabilities().unwrap();
        if let Some(feature) = display.info.mccs_database.get(0x10) {
            display.handle.set_vcp_feature(feature.code, v).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::set_light;

    #[test]
    pub fn t1() {
        let start = Instant::now();
        set_light(20);
        println!("{:?}", start.elapsed());
    }
}
