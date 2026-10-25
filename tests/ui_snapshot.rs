use egui_kittest::Harness;
use tvremote::app::RemoteApp;

#[test]
fn capture_ui_snapshot() {
    let tmp = std::env::temp_dir().join("tvremote_test_snap");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let mut harness = Harness::builder()
        .with_size(eframe::egui::vec2(390.0, 844.0))
        .build_eframe(|cc| {
            RemoteApp::new(cc, tmp)
        });

    harness.run();
    harness.run_steps(4);

    harness.snapshot("remote_ui");
}
