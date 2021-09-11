use kira::{
    arrangement::{
        handle::ArrangementHandle, Arrangement, ArrangementSettings, LoopArrangementSettings,
        SoundClip,
    },
    manager::AudioManager,
    mixer::SubTrackHandle,
    sound::{handle::SoundHandle, SoundSettings},
};

pub struct Audio {
    pub music: SoundHandle,
    pub music_arrangement: ArrangementHandle,
    pub hit: SoundHandle,
    pub hit_arrangement: ArrangementHandle,
    pub music_track: SubTrackHandle,
}

impl Audio {
    #[inline]
    pub fn load(audio_manager: &mut AudioManager) -> ike::anyhow::Result<Self> {
        let music_track = audio_manager.add_sub_track(Default::default())?;

        let music =
            audio_manager.load_sound("assets/audio/OrchardRain.wav", SoundSettings::default())?;

        let mut music_arrangement = audio_manager.add_arrangement(Arrangement::new_loop(
            &music,
            LoopArrangementSettings::default().default_track(music_track.id()),
        ))?;

        music_arrangement.play(Default::default())?;

        let hit = audio_manager.load_sound("assets/audio/hit.wav", SoundSettings::default())?;
        let mut hit_arrangement = Arrangement::new(ArrangementSettings::new());
        hit_arrangement.add_clip(SoundClip::new(&hit, 0.0));

        Ok(Self {
            music_arrangement,
            music,
            hit_arrangement: audio_manager.add_arrangement(hit_arrangement)?,
            hit,
            music_track,
        })
    }
}
