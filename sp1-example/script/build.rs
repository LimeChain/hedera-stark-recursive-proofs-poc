use std::path::PathBuf;

use sp1_helper::build_program;
const EXAMPLES_FOLDER_NAME: &str = "../examples";

fn main() {
    // create examples folder in sp1-example root
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(EXAMPLES_FOLDER_NAME);
    std::fs::create_dir_all(fixture_path).unwrap();
    build_program("../program")
}
