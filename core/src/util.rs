pub fn canonical_to_pos(xy: &str) -> u8 {
    // println!("item: {}", (xy.as_bytes()[1] << 3) as u8 + xy.as_bytes()[0] as u8 - 'a' as u8);
    ((xy.as_bytes()[1] - b'1') << 3) + xy.as_bytes()[0] - b'a'
}

// row number then col number
pub fn pos_to_coord(pos: u8) -> (i8, i8) {
    // println!("a {} {} {}", pos, (pos as i8) << 3, pos as i8 & 0b111);
    ((pos >> 3) as i8, (pos & 0b111) as i8)
}

pub fn coord_to_canonical(coord: (i8, i8)) -> String {
    format!("{}{}", (coord.1 as u8 + b'a') as char, coord.0 + 1)
}

pub fn coord_to_pos(coord: (i8, i8)) -> u8 {
    // println!("b {} {}", coord.0, coord.1);
    ((coord.0 as u8) << 3) + coord.1 as u8
}