use mirajazz::{error::MirajazzError, types::DeviceInput};
use std::sync::Mutex;

/// N1 key count (6x3 = 18: 15 buttons + 3 top LCDs)
const N1_KEY_COUNT: usize = 18;

/// N1 encoder/dial input IDs
/// Input 30: Left face button (above the dial)
/// Input 31: Right face button (above the dial)  
/// Input 35: Dial press (push down on the dial)
/// Input 50: Dial rotation counter-clockwise (left)
/// Input 51: Dial rotation clockwise (right)

/// Track current encoder state [dial_pressed]
static DIAL_PRESSED: Mutex<bool> = Mutex::new(false);

/// Process raw input from N1 device (18 keys: 15 buttons + 3 LCDs, plus dial/face buttons)
/// Device inputs 16-18 (top LCDs) map to OpenDeck keys 0-2 should be working
/// Device inputs 1-15 (main grid) map to OpenDeck keys 3-17
/// Device inputs 30, 31 (face buttons) are remapped to OpenDeck keys 0 and 1
/// Device input 35 (dial press) maps to encoder 0
/// Device inputs 50, 51 (dial rotation) map to encoder 0 twist
pub fn process_input_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    log::info!("Processing N1 input: input={}, state={}", input, state);

    // Handle face buttons (inputs 30, 31) by remapping to top LCD keys (0, 1)
    if input == 30 || input == 31 {
        let remapped_input = if input == 30 { 16 } else { 17 };
        log::info!(
            "N1 face button input {} remapped to virtual input {}",
            input,
            remapped_input
        ); 
        return read_button_press_n1(remapped_input, state);
    }

    // Handle dial press (input 35)
    if input == 35 {
        let is_pressed = state != 0;
        log::info!("N1 dial press: is_pressed={}", is_pressed);
        
        let mut dial_state = DIAL_PRESSED.lock().unwrap();
        *dial_state = is_pressed;
        
        log::info!("→ Sending EncoderStateChange([{}])", is_pressed);
        return Ok(DeviceInput::EncoderStateChange(vec![is_pressed]));
    }

    // Handle dial rotation
    if input == 50 {
        log::info!("N1 dial CCW rotation → EncoderTwist([-1])");
        return Ok(DeviceInput::EncoderTwist(vec![-1]));
    }
    if input == 51 {
        log::info!("N1 dial CW rotation → EncoderTwist([1])");
        return Ok(DeviceInput::EncoderTwist(vec![1]));
    }

    // Handle main buttons (inputs 1-18)
    match input {
        1..=18 => read_button_press_n1(input, state),
        _ => {
            log::warn!("Unknown N1 input {}", input);
            Err(MirajazzError::BadData)
        }
    }
}

fn read_button_states(states: &[u8], key_count: usize) -> Vec<bool> {
    let mut bools = vec![];
    for i in 0..key_count {
        bools.push(states.get(i + 1).copied().unwrap_or(0) != 0);
    }
    bools
}

/// Converts opendeck key index to device key index for N1
/// Maps 3×6 horizontal grid to device inputs.
/// Row 0: [KEY_13][KEY_10][KEY_7][KEY_4][KEY_1][LCD_16]  (OpenDeck 0-5)
/// Row 1: [KEY_14][KEY_11][KEY_8][KEY_5][KEY_2][LCD_17]  (OpenDeck 6-11)
/// Row 2: [KEY_15][KEY_12][KEY_9][KEY_6][KEY_3][LCD_18]  (OpenDeck 12-17)
/// Device requires sending input-1 to address input N (the +1 offset).
pub fn opendeck_to_device(key: u8) -> u8 {
    match key {
        0  => 12,  // → input 13
        1  => 9,   // → input 10
        2  => 6,   // → input 7
        3  => 3,   // → input 4
        4  => 0,   // → input 1
        5  => 15,  // → input 16 (LCD)
        6  => 13,  // → input 14
        7  => 10,  // → input 11
        8  => 7,   // → input 8
        9  => 4,   // → input 5
        10 => 1,   // → input 2
        11 => 16,  // → input 17 (LCD)
        12 => 14,  // → input 15
        13 => 11,  // → input 12
        14 => 8,   // → input 9
        15 => 5,   // → input 6
        16 => 2,   // → input 3
        17 => 17,  // → input 18 (LCD)
        _ => key,
    }
}

/// Converts N1 device key index to opendeck key index
/// Inverse of opendeck_to_device for the 3×6 horizontal layout.
fn device_to_opendeck_n1(key: usize) -> usize {
    match key {
        1  => 4,
        2  => 10,
        3  => 16,
        4  => 3,
        5  => 9,
        6  => 15,
        7  => 2,
        8  => 8,
        9  => 14,
        10 => 1,
        11 => 7,
        12 => 13,
        13 => 0,
        14 => 6,
        15 => 12,
        16 => 5,   // LCD 16
        17 => 11,  // LCD 17
        18 => 17,  // LCD 18
        _ => key.saturating_sub(1),
    }
}

fn read_button_press_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let mut button_states = vec![0x01];
    button_states.extend(vec![0u8; N1_KEY_COUNT + 1]);

    if input == 0 {
        return Ok(DeviceInput::ButtonStateChange(read_button_states(
            &button_states,
            N1_KEY_COUNT,
        )));
    }

    let pressed_index: usize = device_to_opendeck_n1(input as usize);

    if pressed_index < N1_KEY_COUNT {
        button_states[pressed_index + 1] = state;
    }

    Ok(DeviceInput::ButtonStateChange(read_button_states(
        &button_states,
        N1_KEY_COUNT,
    )))
}
