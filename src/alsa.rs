pub struct Audio {
    pub min: i64,
    pub max: i64,
    pub has_switch: bool,
    pub unmuted: bool,
    // pub selem: alsa::mixer::Selem,
}

impl Audio {
    fn new() -> Self {
        Audio {
            min: todo!(),
            max: todo!(),
            has_switch: todo!(),
            unmuted: todo!(),
        }
    }
}
pub fn get_info() {}

#[cfg(test)]
mod test {

    #[test]
    fn test01() {
        let sid = alsa::mixer::SelemId::new("Master", 0);
        let amixer = alsa::mixer::Mixer::new("default", false).unwrap();
        let selem = amixer.find_selem(&sid).unwrap();
        let (min, max) = selem.get_playback_volume_range();
        println!("min: {min} max:{max}");
        let r = selem
            .get_playback_volume(alsa::mixer::SelemChannelId::FrontLeft)
            .unwrap();
        println!("volumn: {r}");
        let can_swith = selem.has_playback_switch();
        println!("can switch: {can_swith}");
        // mute
        selem.set_playback_switch_all(1).unwrap();

        // alsa::mixer::Selem::new(elem)
    }
}
