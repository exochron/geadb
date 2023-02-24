use crate::mount::collect_mounts;

mod mount;
mod tools;

fn main() {
    let mode = "mount";

    match mode {
        "mount" => collect_mounts(),
        &_ => println!("No mode parameter given!"),
    }
}
