use crate::mount::handle_mounts;
use crate::toy::handle_toys;

mod mount;
mod tools;
mod toy;

fn main() {
    let mode = "toy";

    match mode {
        "mount" => handle_mounts(),
        "toy" => handle_toys(),
        &_ => {
            println!("No mode parameter given!")
        }
    };
}
