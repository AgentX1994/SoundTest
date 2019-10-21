/// A Sine / Cos oscillator
///
/// Uses Gordon Smitch difference equation
#[derive(Clone, Debug, Default)]
pub struct SineOscillator {
    /// The omega constant representing the phase change between samples
    omega: f64,
    /// The difference equation coefficient
    epsilon: f64,
    /// Last sine sample
    yn_1: f64,
    /// Last cosine sample
    yqn_1: f64,
    /// frequency generated by this oscillator
    frequency: f64,
    /// Sample rate of the audio stream
    sample_rate: u64,
}

impl SineOscillator {
    pub fn new(frequency: f64, sample_rate: u64) -> Self {
        let mut s = Self::default();
        s.cook_frequency(frequency, sample_rate);
        s.yn_1 = (-s.omega).sin();
        s.yqn_1 = (-2.0 * s.omega).sin();
        s
    }

    pub fn set_frequency(&mut self, frequency: f64) {
        self.cook_frequency(frequency, self.sample_rate);
    }

    pub fn get_frequency(&self) -> f64 {
        self.frequency
    }

    pub fn set_sample_rate(&mut self, sample_rate: u64) {
        self.cook_frequency(self.frequency, sample_rate);
    }

    pub fn get_sample_rate(&self) -> u64 {
        self.sample_rate
    }

    fn cook_frequency(&mut self, frequency: f64, sample_rate: u64) {
        self.sample_rate = sample_rate;
        self.frequency = frequency;
        self.omega = 2.0 * std::f64::consts::PI * frequency / sample_rate as f64;
        self.epsilon = 2.0 * (self.omega / 2.0).sin();
    }

    pub fn step(&mut self) -> (f64, f64) {
        let yq = self.yqn_1 - self.epsilon * self.yn_1;
        let y = self.epsilon * yq + self.yn_1;
        self.yqn_1 = yq;
        self.yn_1 = y;
        (y, yq)
    }
}
