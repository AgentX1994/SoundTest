#![allow(unused_imports)]

use std::cmp::{max, min};
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

use sound_test::filters::biquad::BiquadFilter;
use sound_test::midi::MidiNote;
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

    let keymap: HashMap<Keycode, MidiNote> = [
        (Keycode::Z, MidiNote::new(36)),     // 'z' => C2
        (Keycode::S, MidiNote::new(37)),     // 's' => C#2/Db2
        (Keycode::X, MidiNote::new(38)),     // 'x' => D2
        (Keycode::D, MidiNote::new(39)),     // 'd' => D#2/Eb2
        (Keycode::C, MidiNote::new(40)),     // 'c' => E2
        (Keycode::V, MidiNote::new(41)),     // 'v' => F2
        (Keycode::G, MidiNote::new(42)),     // 'g' => F#2/Gb2
        (Keycode::B, MidiNote::new(43)),     // 'b' => G2
        (Keycode::H, MidiNote::new(44)),     // 'h' => G#2/Ab2
        (Keycode::N, MidiNote::new(45)),     // 'n' => A2
        (Keycode::J, MidiNote::new(46)),     // 'j' => A#2/Bb2
        (Keycode::M, MidiNote::new(47)),     // 'm' => B2
        (Keycode::Comma, MidiNote::new(48)), // ',' => C3
        (Keycode::Q, MidiNote::new(48)),     // 'q' => C3
        (Keycode::Num2, MidiNote::new(49)),  // '2' => C#3/Db3
        (Keycode::W, MidiNote::new(50)),     // 'w' => D3
        (Keycode::Num3, MidiNote::new(51)),  // '3' => D#3/Eb3
        (Keycode::E, MidiNote::new(52)),     // 'e' => E3
        (Keycode::R, MidiNote::new(53)),     // 'r' => F3
        (Keycode::Num5, MidiNote::new(54)),  // '5' => F#3/Gb3
        (Keycode::T, MidiNote::new(55)),     // 't' => G3
        (Keycode::Num6, MidiNote::new(56)),  // '6' => G#3/Ab3
        (Keycode::Y, MidiNote::new(57)),     // 'y' => A3
        (Keycode::Num7, MidiNote::new(58)),  // '7' => A#3/Bb3
        (Keycode::U, MidiNote::new(59)),     // 'u' => B3
        (Keycode::I, MidiNote::new(60)),     // 'i' => C4
        (Keycode::Num9, MidiNote::new(61)),  // '9' => C#4/Db4
        (Keycode::O, MidiNote::new(62)),     // 'o' => D4
        (Keycode::Num0, MidiNote::new(63)),  // '0' => D#4/Eb4
        (Keycode::P, MidiNote::new(64)),     // 'p' => E4
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

    let sample_rate = u64::from(format.sample_rate.0);

    println!("Audio format: {:?}", format);

    let oscs: Arc<Mutex<Vec<WaveTableOscillator>>> = Arc::new(Mutex::new(vec![]));
    let max_polyphony = 16;
    for _ in 0..max_polyphony {
        let osc = WaveTableOscillator::new(sample_rate, SAW_WAVE_TABLE.clone());
        oscs.lock().unwrap().push(osc);
    }

    // For testing purposes
    let mut transpose = 0;

    // Create filter to test with
    let mut lp_filter = BiquadFilter::high_pass(200.0, sample_rate as f64, 0.1);
    println!("{:?}", lp_filter);

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
                        next_value = lp_filter.step(next_value);
                        let value = ((next_value * 0.5 + 0.5) * f64::from(std::u16::MAX)) as u16;
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
                        next_value = lp_filter.step(next_value);
                        let value = (next_value * f64::from(std::i16::MAX)) as i16;
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
                        next_value = lp_filter.step(next_value);
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
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    if !oscs.lock().unwrap().iter().any(|osc| osc.is_playing()) {
                        transpose = min(72, transpose + 12);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    if !oscs.lock().unwrap().iter().any(|osc| osc.is_playing()) {
                        transpose = max(-36, transpose - 12);
                    }
                }
                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    if let Some(ref midinote) = keymap.get(&key) {
                        for osc in oscs.lock().unwrap().iter_mut() {
                            let note = midinote.transpose(transpose);
                            if !osc.is_playing() {
                                println!(
                                    "\tPlaying note {}, frequency {}",
                                    note.note,
                                    note.to_frequency()
                                );
                                osc.note_on(note.to_frequency());
                                break;
                            }
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(midinote) = keymap.get(&key) {
                        for osc in oscs.lock().unwrap().iter_mut() {
                            let note = midinote.transpose(transpose);
                            if osc.is_playing()
                                && (osc.get_frequency() - note.to_frequency()) < std::f64::EPSILON
                            {
                                println!(
                                    "\tStopping note {}, frequency {}",
                                    note.note,
                                    note.to_frequency()
                                );
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
