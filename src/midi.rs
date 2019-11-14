/// A struct representing a MIDI note
#[derive(Debug, Clone, Copy)]
pub struct MidiNote {
    pub note: u8,
}

impl MidiNote {
    pub fn new(note: u8) -> Self {
        MidiNote { note }
    }

    /// Converts this MIDI note into its associated frequency
    pub fn to_frequency(self) -> f64 {
        440.0 * (2.0_f64).powf((f64::from(self.note) - 69.0) / 12.0)
    }

    /// Converts this MIDI note into its associated frequency with the given
    /// A4 base frequency (instead of the default 440 Hz
    pub fn to_frequency_different_tuning(self, a4_tuning: f64) -> f64 {
        a4_tuning * (2.0_f64).powf((f64::from(self.note) - 69.0) / 12.0)
    }

    /// Returns a new MidiNote corresponding to this note transposed by
    /// the given number of semitones
    ///
    /// ```rust
    /// # use sound_test::midi::MidiNote;
    /// let note = MidiNote::new(64);
    /// assert_eq!(note.transpose( 4), MidiNote::new(68));
    /// assert_eq!(note.transpose(-4), MidiNote::new(60));
    pub fn transpose(self, semitones: i8) -> Self {
        MidiNote {
            note: (i16::from(self.note) + i16::from(semitones)) as u8,
        }
    }
}

impl PartialEq for MidiNote {
    fn eq(&self, other: &Self) -> bool {
        self.note == other.note
    }
}
