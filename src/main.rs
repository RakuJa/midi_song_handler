mod audio;
mod hardware_handler;
mod os_explorer;

use crate::hardware_handler::midi_handler::ToggleStates;
use biquad::{Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz, Type};
use dotenv::dotenv;
use hardware_handler::midi_handler::listener_logic;
use ramidier::enums::led_light::color::LedColor;
use ramidier::enums::led_light::mode::LedMode;
use ramidier::enums::message_filter::MessageFilter;
use ramidier::io::output::ChannelOutput;
use rodio::Sink;
use std::env;
use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex};

pub struct State {
    pub music_folder: String,
    pub previous_volume: f32, //used to resume audio volume after mute
    pub previous_pad: u8,
    pub filter: Arc<Mutex<FilterData>>,
    pub music_queue: Sink,
    pub sound_queue: Sink,
    pub button_states: ToggleStates,
}

pub struct FilterData {
    pub previous_filter_percentage: f32,
    pub filter_type: Type<f32>,
    pub filter: Arc<Mutex<DirectForm1<f32>>>,
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    // Setup MIDI Input
    let midi_in = ramidier::io::input::InputChannel::builder()
        .port(2)
        .msg_to_ignore(MessageFilter::None)
        .build()?;

    // Setup MIDI Output
    let mut midi_out = ChannelOutput::builder()
        .port(2)
        .initialize_note_led(true)
        .build()?;
    midi_out.set_all_pads_color(LedMode::On100Percent, LedColor::Off)?;

    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
    let music_sink = Sink::connect_new(stream_handle.mixer());
    let sound_sink = Sink::connect_new(stream_handle.mixer());

    let sample_rate = 44100.0;
    let coeffs = Coefficients::<f32>::from_params(
        Type::AllPass,
        sample_rate.hz(),
        44100.hz(),
        Q_BUTTERWORTH_F32,
    )
    .unwrap();
    let filter_data = Arc::new(Mutex::new(FilterData {
        previous_filter_percentage: 1.,
        filter_type: Type::AllPass,
        filter: Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs))),
    }));
    let music_folder = env::var("MUSIC_FOLDER").unwrap_or_else(|_| "music".to_string());
    let _conn_in = midi_in.listen(
        Some("midir-read-input"),
        move |stamp, rx_data, data| listener_logic(&mut midi_out, stamp, &rx_data, data),
        State {
            music_folder,
            previous_pad: 127,
            previous_volume: 1.0,
            filter: filter_data,
            music_queue: music_sink,
            sound_queue: sound_sink,

            button_states: ToggleStates::default(),
        },
    )?;

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press
    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    match run() {
        Ok(()) => (),
        Err(err) => println!("Error: {err}"),
    }
}
