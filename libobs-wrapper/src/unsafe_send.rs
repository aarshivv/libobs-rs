use libobs::{
    obs_data, obs_display_t, obs_encoder, obs_output, obs_scene_t, obs_sceneitem_t, obs_source, obs_video_info
};
use windows::Win32::{Foundation::{HWND, RECT}, UI::WindowsAndMessaging::GetClientRect};

macro_rules! impl_send_sync {
    ($n:ident, $t:ty) => {
        #[derive(Debug)]
        pub struct $n(pub $t);

        #[cfg(feature = "unsafe-send")]
        unsafe impl Send for $n {}
        #[cfg(feature = "unsafe-send")]
        unsafe impl Sync for $n {}
    };
}

impl_send_sync! { WrappedObsData, *mut obs_data}
impl_send_sync! { WrappedObsOutput, *mut obs_output}
impl_send_sync! { WrappedObsDisplay, *mut obs_display_t}
impl_send_sync! { WrappedObsScene, *mut obs_scene_t}
impl_send_sync! { WrappedObsEncoder, *mut obs_encoder}
impl_send_sync! { WrappedObsVideoInfo, obs_video_info}
impl_send_sync! { WrappedObsSource, *mut obs_source}
impl_send_sync! { WrappedVoidPtr, *mut std::ffi::c_void}
impl_send_sync! { WrappedHWND, HWND }
impl_send_sync! { WrappedObsSceneItem, *mut obs_sceneitem_t }

impl Clone for WrappedObsVideoInfo {
    fn clone(&self) -> Self {
        WrappedObsVideoInfo(self.0.clone())
    }
}

impl PartialEq for WrappedObsVideoInfo {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for WrappedObsVideoInfo {}

impl WrappedHWND {
    pub fn get_window_size(&self) -> (i32, i32) {
        unsafe {
            let mut rect = RECT::default();
            GetClientRect(self.0, &mut rect).unwrap();
            
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            
            (width, height)
        }
    }
}
