use crate::FilterData;
use crate::audio::audio_filter::FilteredSource;
use biquad::{Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz, Type};
use rodio::Sink;
use std::error::Error;
use std::sync::{Arc, Mutex};

pub fn change_filter_frequency_value(
    filter: &Arc<Mutex<FilterData>>,
    value: f32,
    filter_type: Type<f32>,
) {
    let mut data = filter.lock().unwrap();
    let fs = 44100.;
    let next_perc = if data.previous_filter_percentage + value <= 1. {
        1.
    } else {
        data.previous_filter_percentage + value
    };
    let f_val = fs / 100. * next_perc;
    let coeffs = Coefficients::<f32>::from_params(
        filter_type,
        fs.hz(),
        if f_val < fs / 2. { f_val } else { fs / 2. }.hz(),
        Q_BUTTERWORTH_F32,
    )
    .unwrap();
    data.previous_filter_percentage = next_perc;
    data.filter_type = filter_type;


    let mut f = data.filter.lock().unwrap();

    *f = DirectForm1::<f32>::new(coeffs);
}

pub fn change_volume(sink: &Sink, value: f32) -> f32 {
    sink.set_volume(if value <= 0. { 0. } else { value.min(1.) });
    sink.volume()
}

pub fn increase_volume(sink: &Sink, value: f32) -> f32 {
    change_volume(sink, sink.volume() + value)
}

pub fn add_track_to_queue(sink: &Sink, file_path: &str, play: bool) -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::open(file_path)?;
    sink.append(rodio::Decoder::try_from(file)?);
    if play {
        sink.play();
    }
    Ok(())
}

pub fn play_track(
    sink: &Sink,
    file_path: &str,
    filter: &Arc<Mutex<FilterData>>,
) -> Result<(), Box<dyn Error>> {
    sink.stop();
    sink.clear();
    let file = std::fs::File::open(file_path)?;
    let source = rodio::Decoder::try_from(file)?;
    let filtered_source = FilteredSource {
        source,
        filter: Arc::clone(&filter.lock().unwrap().filter),
    };
    sink.append(filtered_source);
    sink.play();

    Ok(())
}
