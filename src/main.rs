#![allow(unused_imports)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use cpal::{StreamData, UnknownTypeOutputBuffer};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use sound_test::oscillator::sine::SineOscillator;
use sound_test::oscillator::wavetable::{
    WaveTable, WaveTableOscillator, SAW_WAVE_TABLE, SINE_WAVE_TABLE, SQUARE_WAVE_TABLE,
    TRIANGLE_WAVE_TABLE,
};

fn main() {
    // Debug output of wave tables
    if let Err(e) = SAW_WAVE_TABLE.dump_to_file("saw.dat") {
        println!("Could not dump saw wave table: {}", e);
    }
    if let Err(e) = SINE_WAVE_TABLE.dump_to_file("sine.dat") {
        println!("Could not dump sine wave table: {}", e);
    }
    if let Err(e) = SQUARE_WAVE_TABLE.dump_to_file("square.dat") {
        println!("Could not dump square wave table: {}", e);
    }
    if let Err(e) = TRIANGLE_WAVE_TABLE.dump_to_file("triangle.dat") {
        println!("Could not dump triangle wave table: {}", e);
    }

    let keymap: HashMap<Keycode, f64> = [
        (Keycode::Z, 65.41),      // 'z' => C2
        (Keycode::S, 69.30),      // 's' => C#2/Db2
        (Keycode::X, 73.42),      // 'x' => D2
        (Keycode::D, 77.78),      // 'd' => D#2/Eb2
        (Keycode::C, 82.41),      // 'c' => E2
        (Keycode::V, 87.31),      // 'v' => F2
        (Keycode::G, 92.50),      // 'g' => F#2/Gb2
        (Keycode::B, 98.00),      // 'b' => G2
        (Keycode::H, 103.83),     // 'h' => G#2/Ab2
        (Keycode::N, 110.00),     // 'n' => A2
        (Keycode::J, 116.54),     // 'j' => A#2/Bb2
        (Keycode::M, 123.47),     // 'm' => B2
        (Keycode::Comma, 130.81), // ',' => C3
        (Keycode::Q, 130.81),     // 'q' => C3
        (Keycode::Num2, 138.59),  // '2' => C#3/Db3
        (Keycode::W, 146.83),     // 'w' => D3
        (Keycode::Num3, 155.56),  // '3' => D#3/Eb3
        (Keycode::E, 164.81),     // 'e' => E3
        (Keycode::R, 174.61),     // 'r' => F3
        (Keycode::Num5, 185.00),  // '5' => F#3/Gb3
        (Keycode::T, 196.00),     // 't' => G3
        (Keycode::Num6, 207.65),  // '6' => G#3/Ab3
        (Keycode::Y, 220.00),     // 'y' => A3
        (Keycode::Num7, 233.08),  // '7' => A#3/Bb3
        (Keycode::U, 246.94),     // 'u' => B3
        (Keycode::I, 261.63),     // 'i' => C4
        (Keycode::Num9, 277.18),  // '9' => C#4/Db4
        (Keycode::O, 293.66),     // 'o' => D4
        (Keycode::Num0, 311.13),  // '0' => D#4/Eb4
        (Keycode::P, 329.63),     // 'p' => E4
    ]
    .iter()
    .copied()
    .collect();

    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

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

    let oscs: Arc<Mutex<Vec<WaveTableOscillator>>> = Arc::new(Mutex::new(vec![]));
    let max_polyphony = 16;
    for _ in 0..max_polyphony {
        let osc = WaveTableOscillator::new(sample_rate, SAW_WAVE_TABLE.clone());
        oscs.lock().unwrap().push(osc);
    }

    let oscs_vec = oscs.clone();
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
                        let mut next_value: f64 = oscs_vec
                            .lock()
                            .unwrap()
                            .iter_mut()
                            .map(&WaveTableOscillator::step)
                            .sum();
                        next_value /= oscs_vec.lock().unwrap().len() as f64;
                        let value = ((next_value * 0.5 + 0.5) * std::u16::MAX as f64) as u16;
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }
                StreamData::Output {
                    buffer: UnknownTypeOutputBuffer::I16(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let mut next_value: f64 = oscs_vec
                            .lock()
                            .unwrap()
                            .iter_mut()
                            .map(&WaveTableOscillator::step)
                            .sum();
                        next_value /= oscs_vec.lock().unwrap().len() as f64;
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
                        let mut next_value: f64 = oscs_vec
                            .lock()
                            .unwrap()
                            .iter_mut()
                            .map(&WaveTableOscillator::step)
                            .sum();
                        next_value /= oscs_vec.lock().unwrap().len() as f64;
                        let value = next_value as f32;
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }
                _ => (),
            }
        });
    });

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => {
                    for osc in oscs.lock().unwrap().iter_mut() {
                        if osc.is_playing() {
                            osc.note_off();
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    if let Some(frequency) = keymap.get(&key) {
                        for osc in oscs.lock().unwrap().iter_mut() {
                            if !osc.is_playing() {
                                println!("\tPlaying frequency {}", frequency);
                                osc.note_on(*frequency);
                                break;
                            }
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(frequency) = keymap.get(&key) {
                        for osc in oscs.lock().unwrap().iter_mut() {
                            if osc.is_playing() && osc.get_frequency() == *frequency {
                                println!("\tStopping frequency {}", frequency);
                                osc.note_off();
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
