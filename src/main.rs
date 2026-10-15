fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(microvmctl::cli::run(&args, &mut std::io::stdout()));
}
