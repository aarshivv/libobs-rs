use std::{
    cmp,
    io::{stdout, Write},
    path::PathBuf,
    process::Command,
    time::Duration,
};

use crate::common::{initialize_obs, test_video};
use libobs_sources::windows::{ObsWindowCaptureMethod, WindowCaptureSourceBuilder, WindowCaptureSourceUpdater};
use libobs_window_helper::{WindowInfo, WindowSearchMode};
use libobs_wrapper::{data::ObsObjectUpdater, unsafe_send::Sendable};
use libobs_wrapper::{
    sources::ObsSourceBuilder,
    utils::{traits::ObsUpdatable, ObsPath},
};

fn find_notepad() -> Option<Sendable<WindowInfo>> {
    let windows =
        WindowCaptureSourceBuilder::get_windows(WindowSearchMode::ExcludeMinimized).unwrap();
    println!("{:?}", windows);
    windows.into_iter().find(|w| {
        w.0.class
            .as_ref()
            .is_some_and(|e| e.to_lowercase().contains("notepad"))
    })
}

#[tokio::test]
// For this test to work, notepad must be open
pub async fn test_window_capture() {
    let rec_file = ObsPath::from_relative("window_capture.mp4").build();
    let path_out = PathBuf::from(rec_file.to_string());

    let mut window = find_notepad();
    if window.is_none() {
        Command::new("notepad.exe").spawn().unwrap();
        std::thread::sleep(Duration::from_millis(350));

        window = find_notepad();
    }

    let window = window.expect("Couldn't find notepad window");

    println!("Recording {:?}", window);

    let (mut context, mut output) = initialize_obs(rec_file).await;
    let mut scene = context.scene("main").await.unwrap();
    scene.set_to_channel(0).await.unwrap();

    let source_name = "test_capture";
    let mut source = context
        .source_builder::<WindowCaptureSourceBuilder, _>(source_name)
        .await
        .unwrap()
        .set_capture_method(ObsWindowCaptureMethod::MethodAuto)
        .set_window(&window)
        .add_to_scene(&mut scene)
        .await
        .unwrap();

    output.start().await.unwrap();
    println!("Recording started");

    let windows = WindowCaptureSourceBuilder::get_windows(WindowSearchMode::ExcludeMinimized)
        .unwrap()
        .into_iter()
        .filter(|e| e.0.obs_id.to_lowercase().contains("code"))
        .collect::<Vec<_>>();
    for i in 0..cmp::min(5, windows.len()) {
        let w = windows.get(i).unwrap();
        println!("Setting to {:?}", w.0.obs_id);

        source
            .create_updater::<WindowCaptureSourceUpdater>()
            .await.unwrap()
            .set_window(w)
            .update()
            .await.unwrap();

        println!("Recording for {} seconds", i);
        stdout().flush().unwrap();
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    println!("Recording stop");

    output.stop().await.unwrap();

    test_video(&path_out, 1.0).await.unwrap();
}
