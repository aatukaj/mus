use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker::*;
use sound::cymbal;
use std::thread;
use std::time::Duration;

pub fn setup() {
    thread::spawn(|| {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let config = device.default_output_config().unwrap();

        match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()).unwrap(),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()).unwrap(),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()).unwrap(),
            _ => panic!("Unsupported format"),
        }
    });
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f64>,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    let mut sequencer = Sequencer::new(false, 1);
    let sequencer_backend = sequencer.backend();

    let mut net = Net::wrap(Box::new(sequencer_backend));
    net = net >> pan(0.0);

    net.set_sample_rate(sample_rate);

    // Use block processing for maximum efficiency.
    let mut backend = BlockRateAdapter::new(Box::new(net.backend()));

    let mut next_value = move || backend.get_stereo();

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;
    let pitch_hz = midi_hz(50.0);
    let pitch =
        lfo(move |t| pitch_hz * xerp11(1.0 / (1.0), 1.0, 0.5 * (sin_hz(6.0, t) + sin_hz(6.1, t))));

    sequencer.push_duration(
        0.0,
        2.0,
        Fade::Smooth,
        0.02,
        0.2,
        Box::new((pitch >> square()) * 0.5),
    );
    thread::sleep(Duration::from_secs(5));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f64>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left: T = T::from_sample(sample.0 as f64);
        let right: T = T::from_sample(sample.1 as f64);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
