use gilrs::{Axis, Button};

/// Function to translate boolean button values into the bytes that the roboRIO expects
pub fn to_u8_vec(vec_in: &[bool]) -> Vec<u8> {
    let mut vec = Vec::new();

    for i in (0..vec_in.len()).step_by(8) {
        let mut num: u8 = 0;
        for j in i..i+8 {
            num <<= 1;
            num |= *match vec_in.get(j) {
                Some(a) => a,
                None    => &false,
            } as u8;
        }
        vec.push(num);
    }

    vec
}

pub fn ip_from_team_number(team: u32) -> String {
    let s = team.to_string();

    match s.len() {
        1 | 2 => format!("10.0.{}.2", team),
        3 => format!("10.{}.{}.2", &s[0..1], &s[1..3]),
        4 => format!("10.{}.{}.2", &s[0..2], &s[2..4]),
        _ => unreachable!()
    }
}

pub fn axis_to_roborio(axis: Axis) -> u8 {
    match axis {
        Axis::LeftStickX => 0,
        Axis::LeftStickY => 1,
        Axis::RightStickX => 4,
        Axis::RightStickY => 5,
        _ => unreachable!()
    }
}

pub fn button_to_roborio(button: Button) -> u8 {
    match button {
        Button::LeftTrigger2 => 2,
        Button::RightTrigger2 => 3,
        Button::South => 1,
        Button::East => 2,
        Button::West => 3,
        Button::North => 4,
        Button::LeftTrigger => 5,
        Button::RightTrigger => 6,
        Button::Select => 7,
        Button::Start => 8,
        Button::LeftThumb => 9,
        Button::RightThumb => 10,
        _ => unimplemented!()
    }
}