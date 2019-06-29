use chrono::Local;
use log::LevelFilter;
use std::path::Path;

fn init() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .is_test(true)
        .init();
}

#[cfg(feature = "mock_profiles")]
#[test]
fn run() {
    init();

    let profile_path = format!("/tmp/viska-test/{}", Local::now().timestamp_millis());
    viska::mock_profiles::new_mock_profile(&Path::new(&profile_path));
}
