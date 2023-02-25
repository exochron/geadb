use crate::mount::handle_mounts;

mod mount;
mod tools;

fn main() {
    let mode = "mount";

    match mode {
        "mount" => handle_mounts(),
        &_ => {
            println!("No mode parameter given!")
        }
    };
}
