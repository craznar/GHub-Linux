use llc_service::*;
fn run() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|a| a == "--version") {
        println!("llc-service {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let backend = Backend::default();
    let state = backend.state.clone();
    let connection = zbus::blocking::connection::Builder::system()?
        .method_timeout(std::time::Duration::from_secs(2))
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, backend)?
        .build()?;
    eprintln!("Linux Logitech Control: passive backend ready; hardware gate G0 remains unverified");
    loop {
        std::thread::sleep(std::time::Duration::from_millis(250));
        supervise(&connection, &state);
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("llc-service: {error}");
        std::process::exit(4);
    }
}
