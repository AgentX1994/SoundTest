use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use cpal::{StreamData, UnknownTypeOutputBuffer};

use sound_test::oscillator::sine::SineOscillator;
use sound_test::oscillator::wavetable::{WaveTable, WaveTableOscillator, SAW_WAVE_TABLE};

fn main() {
    // Get the default audio host
    let host = cpal::default_host();
    let event_loop = host.event_loop();

    // Get the default audio device from the host
    let device = host
        .default_output_device()
        .expect("no output device available");

    // Get a format
    let format = device
        .default_output_format()
        .expect("No formats supported!");

    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();

    event_loop
        .play_stream(stream_id)
        .expect("failed to play stream");

    let sample_rate = format.sample_rate.0 as u64;

    println!("Audio format: {:?}", format);

    let mut oscs: Vec<WaveTableOscillator> = vec![];
    for i in 0..format.channels {
        let osc = WaveTableOscillator::new(
            500.0 + 100.0 * (i as f64),
            sample_rate,
            SAW_WAVE_TABLE.clone(),
        );
        println!("Osc {}: {:?}\n", i, osc);
        oscs.push(osc);
    }

    thread::spawn(move || {
        event_loop.run(move |stream_id, stream_result| {
            let stream_data = match stream_result {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("an error occured on stream {:?}: {}", stream_id, err);
                    return;
                }
            };

            match stream_data {
                StreamData::Output {
                    buffer: UnknownTypeOutputBuffer::U16(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let next_values: Vec<f64> =
                            oscs.iter_mut().map(&WaveTableOscillator::step).collect();
                        let mut values = next_values
                            .iter()
                            .map(|x| ((x * 0.5 + 0.5) * std::u16::MAX as f64) as u16);
                        for out in sample.iter_mut() {
                            match values.next() {
                                Some(x) => *out = x,
                                None => *out = std::u16::MAX / 2,
                            }
                        }
                    }
                }
                StreamData::Output {
                    buffer: UnknownTypeOutputBuffer::I16(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let next_values: Vec<f64> =
                            oscs.iter_mut().map(&WaveTableOscillator::step).collect();

                        let mut values = next_values
                            .iter()
                            .map(|x| (x * std::i16::MAX as f64) as i16);
                        for out in sample.iter_mut() {
                            match values.next() {
                                Some(x) => *out = x,
                                None => *out = 0,
                            }
                        }
                    }
                }
                StreamData::Output {
                    buffer: UnknownTypeOutputBuffer::F32(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let next_values: Vec<f64> =
                            oscs.iter_mut().map(&WaveTableOscillator::step).collect();

                        let mut values = next_values.iter().map(|x| *x as f32);
                        for out in sample.iter_mut() {
                            match values.next() {
                                Some(x) => *out = x,
                                None => *out = 0.0,
                            }
                        }
                    }
                }
                _ => (),
            }

            oscs.iter_mut().for_each(|osc| {
                osc.set_frequency(osc.get_frequency() * 1.01);
                if osc.get_frequency() > (osc.get_sample_rate() / 2) as f64 {
                    osc.set_frequency(osc.get_frequency() - (osc.get_sample_rate() / 2) as f64);
                }
            });
        });
    });

    thread::sleep(Duration::new(10, 0));
}
