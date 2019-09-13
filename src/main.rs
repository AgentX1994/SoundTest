use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use cpal::{StreamData, UnknownTypeOutputBuffer};

use sound_test::oscillator::saw::SawOscillator;
use sound_test::oscillator::sine::SineOscillator;

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

    let mut osc = SawOscillator::new(500.0, sample_rate);
    println!("{:?}", osc);

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
                    //let next_value = osc.step().0;
                    let next_value = osc.step();
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let value = ((next_value * 0.5 + 0.5) * std::u16::MAX as f64) as u16;
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }
                StreamData::Output {
                    buffer: UnknownTypeOutputBuffer::I16(mut buffer),
                } => {
                    //let next_value = osc.step().0;
                    let next_value = osc.step();
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let value = (next_value * std::i16::MAX as f64) as i16;
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }
                StreamData::Output {
                    buffer: UnknownTypeOutputBuffer::F32(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        //let next_value = osc.step().0;
                        let next_value = osc.step();
                        let value = next_value as f32;
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }
                _ => (),
            }
            osc.set_frequency(osc.get_frequency() * 1.01);
            if osc.get_frequency() > (osc.get_sample_rate() / 2) as f64 {
                osc.set_frequency(osc.get_frequency() - (osc.get_sample_rate() / 2) as f64);
            }
        });
    });

    thread::sleep(Duration::new(10, 0));
}
