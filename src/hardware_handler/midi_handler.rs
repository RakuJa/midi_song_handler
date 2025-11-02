use crate::State;
use crate::audio::playback_handler;
use crate::os_explorer::explorer::search_files_in_path;
use biquad::Type;
use log::{debug, info, warn};
use ramidier::enums::button::input_group::InputGroup;
use ramidier::enums::button::knob_ctrl::KnobCtrlKey;
use ramidier::enums::button::pads::PadKey;
use ramidier::enums::button::soft_keys::SoftKey;
use ramidier::enums::led_light::color::LedColor;
use ramidier::enums::led_light::mode::LedMode;
use ramidier::io::input_data::MidiInputData;
use ramidier::io::output::ChannelOutput;

#[derive(Debug, Default)]
pub struct ToggleStates {
    pub is_clip_stop_on: bool,
    pub is_solo_on: bool,
    pub is_mute_on: bool,
    pub is_rec_arm_on: bool,
    pub is_select_on: bool,
    pub is_stop_all_on: bool,
    pub is_volume_on: bool,
    pub is_pan_on: bool,
    pub is_send_on: bool,
    pub is_device_on: bool,
    pub is_shift_on: bool,
    pub is_filter_on: bool,
    pub is_start_on: bool,
}

fn handle_pad(pad: PadKey, data: &mut State, midi_out: &mut ChannelOutput) {
    let note = pad.get_index();

    // Turn off previous pad
    let _ = midi_out.set_pad_led(LedMode::On10Percent, data.previous_pad, LedColor::Off);

    data.previous_pad = note;
    let prefix = format!("{note:02}_");
    if let Ok(res) = search_files_in_path(data.music_folder.as_str(), prefix.as_str()) {
        info!("playing the following audio folder: {:?}", res.0);
        let files = res.1;
        if let Some(first_track) = files.first() {
            let () = &data.music_queue.clear();
            playback_handler::play_track(&data.music_queue, first_track.as_str(), &data.filter)
                .unwrap();
            files.iter().skip(1).for_each(|file| {
                playback_handler::add_track_to_queue(&data.music_queue, file.as_str(), false)
                    .unwrap();
            });
        }
    } else {
        warn!("No folder associated with the given button {note}");
    }
    let color: LedColor = (pad.get_index() + 1).try_into().unwrap_or(LedColor::Green);
    let _ = midi_out.set_pad_led(LedMode::On100Percent, note, color);
}

fn handle_knob(index: u8, value: u8, data: &State) {
    let delta = if value > 63 { -1.0 } else { 1.0 };

    match index {
        1 => {
            if !data.button_states.is_mute_on {
                playback_handler::increase_volume(&data.music_queue, delta * 0.01);
            }
        }
        8 => playback_handler::change_filter_frequency_value(&data.filter, delta, Type::LowPass),
        _ => {}
    }
}

fn handle_resume_pause(data: &mut State, midi_out: &mut ChannelOutput) {
    toggle_button(
        &mut data.button_states.is_filter_on,
        midi_out,
        InputGroup::ResumePause,
        LedColor::Green,
    );

    let filter_type = if data.button_states.is_filter_on {
        playback_handler::change_filter_frequency_value(&data.filter, 1., Type::LowPass);
        Type::LowPass
    } else {
        playback_handler::change_filter_frequency_value(&data.filter, 0., Type::AllPass);
        Type::AllPass
    };

    playback_handler::change_filter_frequency_value(&data.filter, 1., filter_type);
}

fn handle_soft_key(key: SoftKey, data: &mut State, midi_out: &mut ChannelOutput) {
    match key {
        SoftKey::ClipStop => toggle_button(
            &mut data.button_states.is_clip_stop_on,
            midi_out,
            key,
            LedColor::Green,
        ),
        SoftKey::Solo => toggle_button(
            &mut data.button_states.is_solo_on,
            midi_out,
            key,
            LedColor::Green,
        ),
        SoftKey::Mute => {
            toggle_button(
                &mut data.button_states.is_mute_on,
                midi_out,
                key,
                LedColor::Green,
            );
            if data.button_states.is_mute_on {
                data.previous_volume = data.music_queue.volume();
                playback_handler::change_volume(&data.music_queue, 0.);
            } else {
                playback_handler::change_volume(&data.music_queue, data.previous_volume);
            }
        }
        SoftKey::RecArm => toggle_button(
            &mut data.button_states.is_rec_arm_on,
            midi_out,
            key,
            LedColor::Green,
        ),
        SoftKey::Select => toggle_button(
            &mut data.button_states.is_select_on,
            midi_out,
            key,
            LedColor::Green,
        ),
    }
}

fn handle_knob_ctrl(key: KnobCtrlKey, data: &mut State, midi_out: &mut ChannelOutput) {
    match key {
        KnobCtrlKey::Volume => toggle_button(
            &mut data.button_states.is_volume_on,
            midi_out,
            key,
            LedColor::Green,
        ),
        KnobCtrlKey::Pan => toggle_button(
            &mut data.button_states.is_pan_on,
            midi_out,
            key,
            LedColor::Green,
        ),
        KnobCtrlKey::Send => {
            toggle_button(
                &mut data.button_states.is_send_on,
                midi_out,
                key,
                LedColor::Green,
            );
        }
        KnobCtrlKey::Device => toggle_button(
            &mut data.button_states.is_device_on,
            midi_out,
            key,
            LedColor::Green,
        ),
    }
}

/// Generic toggle helper for buttons or softkeys
fn toggle_button<T: Into<u8> + Copy>(
    state: &mut bool,
    midi_out: &mut ChannelOutput,
    key: T,
    color: LedColor,
) {
    *state = !*state;
    change_button_status(midi_out, *state, key.into(), color);
}

pub fn listener_logic(
    midi_out: &mut ChannelOutput,
    stamp: u64,
    msg: &MidiInputData,
    data: &mut State,
) {
    debug!("{stamp}: {msg:?}");
    if msg.value > 0 {
        match msg.input_group {
            InputGroup::Pads(ref pad) => handle_pad(*pad, data, midi_out),
            InputGroup::Knob(index) => handle_knob(index, msg.value, data),
            InputGroup::ResumePause => handle_resume_pause(data, midi_out),
            InputGroup::SoftKeys(ref key) => handle_soft_key(*key, data, midi_out),
            InputGroup::KnobCtrl(ref key) => handle_knob_ctrl(*key, data, midi_out),
            InputGroup::StopAllClips => toggle_button(
                &mut data.button_states.is_stop_all_on,
                midi_out,
                InputGroup::StopAllClips,
                LedColor::Green,
            ),
            InputGroup::Shift => toggle_button(
                &mut data.button_states.is_shift_on,
                midi_out,
                InputGroup::Shift,
                LedColor::Green,
            ),
            InputGroup::Start => toggle_button(
                &mut data.button_states.is_start_on,
                midi_out,
                InputGroup::Start,
                LedColor::Green,
            ),
            InputGroup::Left => {
                change_button_status(midi_out, true, InputGroup::Left, LedColor::Green);
            }
            InputGroup::Right => {
                change_button_status(midi_out, true, InputGroup::Right, LedColor::Green);
                data.music_queue.skip_one();
            }
            InputGroup::Up => change_button_status(midi_out, true, InputGroup::Up, LedColor::Green),
            InputGroup::Down => {
                change_button_status(midi_out, true, InputGroup::Down, LedColor::Green);
            }
        }
    } else {
        match msg.input_group {
            InputGroup::Left => {
                change_button_status(midi_out, false, InputGroup::Left, LedColor::Green);
            }
            InputGroup::Right => {
                change_button_status(midi_out, false, InputGroup::Right, LedColor::Green);
            }
            InputGroup::Up => {
                change_button_status(midi_out, false, InputGroup::Up, LedColor::Green);
            }
            InputGroup::Down => {
                change_button_status(midi_out, false, InputGroup::Down, LedColor::Green);
            }
            _ => {}
        }
    }
}

fn change_button_status<T>(
    midi_out: &mut ChannelOutput,
    next_state: bool,
    button_index: T,
    color: LedColor,
) where
    T: Into<u8>,
{
    let _ = midi_out.set_pad_led(
        LedMode::On100Percent,
        button_index,
        if next_state { color } else { LedColor::Off },
    );
}
