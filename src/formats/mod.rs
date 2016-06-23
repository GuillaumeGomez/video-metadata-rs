use std::io::Read;
use enums;

macro_rules! check_format {
    ($m:ident, $f:expr, $possibilities:expr) => {
        match $m::get_information($f) {
            enums::Result::Unknown => {}
            enums::Result::Incomplete(x) => {
                $possibilities.push(x[0].clone());
            }
            x => return x,
        }
    }
}

pub fn get_format<T: Read>(f: &mut T) -> enums::Result {
    let mut possibilities = vec!();
    let mut input = vec!();
    f.read_to_end(&mut input);

    check_format!(webm, &input, possibilities);
    enums::Result::Incomplete(possibilities)
}

mod webm;
