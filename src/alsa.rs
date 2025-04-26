pub struct Audio {
    pub min: i64,
    pub max: i64,
    pub has_switch: bool,
    mixer: alsa::mixer::Mixer,
    sid: alsa::mixer::SelemId,
}

impl Audio {
    pub fn new() -> Self {
        let sid = alsa::mixer::SelemId::new("Master", 0);
        let mixer = alsa::mixer::Mixer::new("default", false).unwrap();
        let selem = mixer.find_selem(&sid).unwrap();

        let (min, max) = selem.get_playback_volume_range();
        let has_switch = selem.has_playback_switch();
        // let unmuted = match selem.get_playback_switch(alsa::mixer::SelemChannelId::FrontLeft) {
        //     Ok(1) => true,
        //     _ => false,
        // };

        Audio {
            min,
            max,
            has_switch,
            mixer,
            sid,
        }
    }
    fn get_selem(&self) -> alsa::mixer::Selem {
        self.mixer.find_selem(&self.sid).unwrap()
    }
    pub fn get_current_volume(&self) -> f64 {
        let selem = self.get_selem();
        let v = selem
            .get_playback_volume(alsa::mixer::SelemChannelId::FrontLeft)
            .unwrap();
        v as f64 / (self.max - self.min) as f64
    }
    pub fn is_unmuted(&self) -> bool {
        let selem = self.get_selem();
        selem
            .get_playback_switch(alsa::mixer::SelemChannelId::FrontLeft)
            .unwrap()
            == 1
    }
    pub fn set_current_volumn(&self, v: f64) {
        let mut v = v.max(0.);
        v = v.min(1.);
        let v: i64 = (v * (self.max - self.min) as f64).floor() as i64;
        let selem = self.get_selem();
        selem.set_playback_volume_all(v).unwrap();
    }
    pub fn toggle_mute(&self) {
        let selem = self.get_selem();
        selem
            .set_playback_switch_all(match self.is_unmuted() {
                true => 0,
                false => 1,
            })
            .unwrap();
    }
}

impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_info() {}

#[cfg(test)]
mod test {
    use super::Audio;

    #[test]
    fn test01() {
        let mut audio = Audio::new();
        println!("{}", audio.get_current_volume());
        audio.set_current_volumn(0.24);
        let u = audio.is_unmuted();
        println!("{u}");
        audio.toggle_mute();
        assert_eq!(audio.is_unmuted(), !u);
    }
}
