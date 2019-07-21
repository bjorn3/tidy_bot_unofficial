mod check;

use check::*;

fn main() {
    clone_repo(
        "https://github.com/bjorn3/rust",
        "709120b32146e74c19ecb53fd58b2b108fa9096a",
    )
    .unwrap();

    for error in run_tidy() {
        if let Some((file, line)) = error.file_and_line {
            print!("{:<32} {:<4}: ", file, line);
        } else {
            print!("{:<37}: ", "<unknown>");
        }
        println!("{}", error.message);
    }
}
