fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("tidewm.ico");
    res.compile().expect("Failed to embed icon");
    println!("cargo:rerun-if-changed=tidewm.ico");
}
