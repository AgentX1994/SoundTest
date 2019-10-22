use std::default;
/// # Biquad Filter
///
/// Implements a "Biquad" (Biquadratic) filter with the transfer function
///
/// H(z) = (b0 + b1 * xn_1 + b2 * xn_2) / (1 + a1 * yn_1 + a2 * yn_2)
///
/// The difference equation is
///
/// y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2]  - a1*y[n-1] - a2*y[n-2]
///
#[derive(Debug)]
pub struct BiquadFilter {
    /// Filter coefficients
    /// b cofficients are for the input, and a is for the output
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,

    /// delay registers
    /// x is "feed-forward", y is "feed-back"
    /// n_1 means previous sample, and n_2 means two samples ago
    xn_1: f64,
    xn_2: f64,
    yn_1: f64,
    yn_2: f64,
}

impl default::Default for BiquadFilter {
    /// Creates a default BiquadFilter with everything zeroed
    fn default() -> Self {
        BiquadFilter {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            xn_1: 0.0,
            xn_2: 0.0,
            yn_1: 0.0,
            yn_2: 0.0,
        }
    }
}

impl BiquadFilter {
    /// Creates a new BiquadFilter with the given coefficients
    pub fn new(b0: f64, b1: f64, b2: f64, a1: f64, a2: f64) -> BiquadFilter {
        BiquadFilter {
            b0,
            b1,
            b2,
            a1,
            a2,
            xn_1: 0.0,
            xn_2: 0.0,
            yn_1: 0.0,
            yn_2: 0.0,
        }
    }

    /// Creates a BiquadFilter set up as a low pass filter with the given frequency at the given
    /// sample rate, and with the given quality (Controls how sharp the cutoff is)
    pub fn low_pass(frequency: f64, sample_rate: f64, quality: f64) -> BiquadFilter {
        let omega_naught = 2.0 * std::f64::consts::PI * frequency / sample_rate;
        let alpha = omega_naught.sin() / (2.0 * quality);
        let cos_omega_naught = omega_naught.cos();
        let cos_omega_naught_prime = 1.0 - cos_omega_naught;
        let cos_omega_naught_prime_div_2 = cos_omega_naught_prime / 2.0;

        BiquadFilter {
            b0: cos_omega_naught_prime_div_2 / (1.0 + alpha),
            b1: cos_omega_naught_prime / (1.0 + alpha),
            b2: cos_omega_naught_prime_div_2 / (1.0 + alpha),
            a1: -2.0 * cos_omega_naught / (1.0 + alpha),
            a2: (1.0 - alpha) / (1.0 + alpha),

            xn_1: 0.0,
            xn_2: 0.0,
            yn_1: 0.0,
            yn_2: 0.0,
        }
    }

    /// Creates a BiquadFilter set up as a high pass filter with the given frequency at the given
    /// sample rate, and with the given quality (Controls how sharp the cutoff is)
    pub fn high_pass(frequency: f64, sample_rate: f64, quality: f64) -> BiquadFilter {
        let omega_naught = 2.0 * std::f64::consts::PI * frequency / sample_rate;
        let alpha = omega_naught.sin() / (2.0 * quality);
        let cos_omega_naught = omega_naught.cos();
        let cos_omega_naught_prime = cos_omega_naught + 1.0;
        let cos_omega_naught_prime_div_2 = cos_omega_naught_prime / 2.0;

        BiquadFilter {
            b0: cos_omega_naught_prime_div_2 / (1.0 + alpha),
            b1: -cos_omega_naught_prime / (1.0 + alpha),
            b2: cos_omega_naught_prime_div_2 / (1.0 + alpha),
            a1: -2.0 * cos_omega_naught / (1.0 + alpha),
            a2: (1.0 - alpha) / (1.0 + alpha),

            xn_1: 0.0,
            xn_2: 0.0,
            yn_1: 0.0,
            yn_2: 0.0,
        }
    }

    /// Steps the filter using the given input sample, and returns the next output sample
    pub fn step(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.b1 * self.xn_1 + self.b2 * self.xn_2
            - self.a1 * self.yn_1
            - self.a2 * self.yn_2;

        self.xn_2 = self.xn_1;
        self.xn_1 = x;

        self.yn_2 = self.yn_1;
        self.yn_1 = y;

        y
    }

    /// Steps the filter using the given input samples, and returns the corresponding output
    /// samples
    pub fn step_buffer(&mut self, xs: &[f64]) -> Vec<f64> {
        let mut ys = std::vec::Vec::with_capacity(xs.len());

        for x in xs {
            ys.push(self.step(*x));
        }

        ys
    }
}
