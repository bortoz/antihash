pub fn find_collision(length: usize) -> Option<(String, String)> {
    let mut fi = String::with_capacity(length);
    let mut se = String::with_capacity(length);
    for i in 0..length {
        let p = (i.count_ones() % 2) as u8;
        fi.push(char::from(97 + p));
        se.push(char::from(98 - p));
    }
    Some((fi, se))
}
