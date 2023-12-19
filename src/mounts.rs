use crate::mount::handle_mounts;
use crate::tools::ProductVersion;

mod mount;
mod tools;

fn main() {
    handle_mounts(ProductVersion::determine_from_cli());
}
