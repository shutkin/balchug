use balchug_common::scenario::Scenario;
use crate::project::create_animations;

pub mod project;

fn main() {
    let scenario = Scenario {
        sprites: create_animations(),
    };
    println!("{scenario:?}");
}
