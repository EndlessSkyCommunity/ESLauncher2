#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("icons/icon.ico");
    res.set_language(0x0009);
    res.set("ProductName", "ESLauncher2");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {}
