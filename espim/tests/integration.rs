use std::fs;

#[test]
fn install_wf() {
    let plugins = espim::retrieve_plugins().unwrap();
    let mut wf = plugins
        .iter()
        .find(|p| p.name().eq_ignore_ascii_case("world forge"))
        .unwrap()
        .clone();
    wf.download().unwrap();
    let path = wf.path().unwrap();

    let (available_version, installed_version) = wf.versions();
    assert_eq!(available_version.unwrap(), installed_version.unwrap(),);

    let mut version_file = path.clone();
    version_file.push(".version");
    assert_eq!(
        installed_version.unwrap(),
        fs::read_to_string(version_file).unwrap()
    );

    wf.remove().unwrap();
    assert!(!path.exists());
}
