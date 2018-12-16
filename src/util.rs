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
        vec.push(reverse_byte(num));
    }

    vec.into_iter().rev().collect()
}

fn reverse_byte(mut byte: u8) -> u8 {
    byte = (byte & 0xF0) >> 4 | (byte & 0x0F) << 4;
    byte = (byte & 0xCC) >> 2 | (byte & 0x33) << 2;
    byte = (byte & 0xAA) >> 1 | (byte & 0x55) << 1;

    byte
}

/// Converts the given team number into a String containing the IP of the roboRIO
/// Assumes the roboRIO will exist at 10.TE.AM.2
pub fn ip_from_team_number(team: u32) -> String {
    let s = team.to_string();

    match s.len() {
        1 | 2 => format!("10.0.{}.2", team),
        3 => format!("10.{}.{}.2", &s[0..1], &s[1..3]),
        4 => format!("10.{}.{}.2", &s[0..2], &s[2..4]),
        _ => unreachable!() // Team numbers shouldn't be >4 characters
    }
}
