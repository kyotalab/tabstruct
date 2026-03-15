fn main() {
    if let Err(err) = tabstruct::app::run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
