fn main() {
    let (major, minor, patch) = anyhal::VERSION;
    println!("AnyHAL {major}.{minor}.{patch} (host)");
}
