

pub fn to_u8_vec(vec_in: &Vec<bool>) -> Vec<u8> {
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