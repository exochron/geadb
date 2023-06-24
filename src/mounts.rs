use crate::mount::handle_mounts;
use crate::tools::determine_game_version_from_cli;

mod mount;
mod tools;

fn main() {
    handle_mounts(determine_game_version_from_cli());
}
