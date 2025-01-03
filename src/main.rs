mod skyutils;
mod skyfigures;
mod skyvariants;
mod skyhats;

use skyutils::Skylander;
use skyfigures::{Character, IntoEnumIterator};
use skyvariants::Variant;
use skyhats::Hat;

fn main() {
    let sky1 = Skylander::new(Character::TriggerHappy, Variant::Series3, Some([0x20, 0x24, 0x49, 0x12]));
    sky1.save_to_filename("../Skylanders_Files/Tests/test1.sky").expect("couldn't save file");

    for c in Character::iter() {
        println!("{}", c.to_string());
    }

    for h in Hat::iter() {
        println!("{}", h.to_string());
    }
}