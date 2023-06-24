use crate::tools::determine_game_version_from_cli;
use crate::toy::handle_toys;

mod tools;
mod toy;

pub fn main() {
    handle_toys(determine_game_version_from_cli());
}
