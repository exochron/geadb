fn main() {
    let mode = "mount";

    match mode {
        "mount" => {}
        &_ => println!("No mode parameter given!"),
    }
}
