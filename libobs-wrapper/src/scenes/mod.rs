use std::{
    cell::RefCell,
    rc::Rc,
};

use getters0::Getters;
use libobs::{obs_scene_create, obs_scene_t, obs_set_output_source, obs_source_t};

use crate::{
    context::ObsContextShutdownZST, enums::Vec2, sources::{ObsFilterRef, ObsSourceRef}, unsafe_send::{WrappedObsScene, WrappedObsSceneItem}, utils::{ObsError, ObsPath, ObsString, SourceInfo}
};

#[derive(Debug)]
struct _SceneDropGuard {
    scene: WrappedObsScene,
}

impl Drop for _SceneDropGuard {
    fn drop(&mut self) {
        unsafe {
            libobs::obs_scene_release(self.scene.0);
        }
    }
}

#[derive(Debug, Clone, Getters)]
#[skip_new]
pub struct ObsSceneRef {
    #[skip_getter]
    scene: Rc<WrappedObsScene>,
    name: ObsString,
    #[get_mut]
    pub(crate) sources: Rc<RefCell<Vec<(ObsSourceRef, WrappedObsSceneItem)>>>,
    #[skip_getter]
    pub(crate) active_scene: Rc<RefCell<Option<WrappedObsScene>>>,

    #[skip_getter]
    _guard: Rc<_SceneDropGuard>,

    #[skip_getter]
    _shutdown: Rc<ObsContextShutdownZST>,
}

impl ObsSceneRef {
    pub(crate) fn new(name: ObsString, active_scene: Rc<RefCell<Option<WrappedObsScene>>>, shutdown: Rc<ObsContextShutdownZST>) -> Self {
        let scene = unsafe { obs_scene_create(name.as_ptr()) };

        Self {
            name,
            scene: Rc::new(WrappedObsScene(scene)),
            sources: Rc::new(RefCell::new(vec![])),
            active_scene: active_scene.clone(),
            _guard: Rc::new(_SceneDropGuard {
                scene: WrappedObsScene(scene),
            }),
            _shutdown: shutdown
        }
    }

    pub fn add_and_set(&self, channel: u32) {
        let mut s = self.active_scene.borrow_mut();
        *s = Some(WrappedObsScene(self.as_ptr()));

        unsafe {
            obs_set_output_source(channel, self.get_scene_source_ptr());
        }
    }

    pub fn get_scene_source_ptr(&self) -> *mut obs_source_t {
        unsafe { libobs::obs_scene_get_source(self.scene.0) }
    }

    pub fn add_source(&mut self, info: SourceInfo) -> Result<ObsSourceRef, ObsError> {
        let source = ObsSourceRef::new(info.id, info.name, info.settings, info.hotkey_data);

        return match source {
            Ok(x) => {
                let scene_item =unsafe {
                    libobs::obs_scene_add(self.scene.0, x.source.0)
                };
                let tmp = x.clone();
                self.sources.borrow_mut().push((x, WrappedObsSceneItem(scene_item)));
                Ok(tmp)
            }
            Err(x) => Err(x),
        };
    }

    pub fn get_source_by_index(&self, index: usize) -> Option<ObsSourceRef> {
        self.sources.borrow().get(index).map(|x| x.0.clone())
    }

    pub fn get_source_mut(&self, name: &str) -> Option<ObsSourceRef> {
        self.sources
        .borrow()
            .iter()
            .find(|x| x.0.name() == name)
            .map(|x| x.0.clone())
    }

    pub fn remove_source(&mut self, name: ObsString) -> Result<(), ObsError> {
        // Find the source by name
        let index =if let Some(index) = self.sources.borrow().iter().position(|x| x.0.name == name) {
            unsafe {
                // Find the scene item for this source
                let scene_item = libobs::obs_scene_find_source(self.scene.0, name.as_ptr());
                if !scene_item.is_null() {
                    // Remove the scene item
                    libobs::obs_sceneitem_remove(scene_item);
                    // Release the scene item reference
                    libobs::obs_sceneitem_release(scene_item);
                }
            }
            
            index
        } else {
            return Err(ObsError::SourceNotFound)
        };

        // Remove from our sources list
        self.sources.borrow_mut().remove(index);
        Ok(())
    }

    pub fn enable_source(&self, name: &str, enable: bool) -> Result<(), ObsError> {
        match self.sources.borrow().iter().find(|x| x.0.name() == name) {
            Some(x) => {
                unsafe {
                    if libobs::obs_source_enabled(x.0.source.0) != enable {
                        libobs::obs_source_set_enabled(x.0.source.0, enable);
                    } else {
                        return Err(ObsError::SourceNotFound);
                    }
                };
                Ok(())
            }
            None => Err(ObsError::SourceNotFound),
        }
    }

    /// Make sure source with name `source_name` is present along with source with name `filter_name`
    pub fn add_source_filter(&self, source_name: &str, filter_ref: &ObsFilterRef) -> Result<(), ObsError> {
        match self.sources
                .borrow()
                .iter()
                .find(|x| x.0.name() == source_name) {
            Some(source) => {
                unsafe {
                    libobs::obs_source_filter_add(source.0.source.0, filter_ref.source.0);
                }
                Ok(())
            }
            _ => Err(ObsError::SourceNotFound),
        }
    }

    /// Make sure source with name `source_name` is present along with source with name `filter_name`
    pub fn remove_source_filter(&self, source_name: &str, filter_ref: &ObsFilterRef) -> Result<(), ObsError> {
        match self.sources
                .borrow()
                .iter()
                .find(|x| x.0.name() == source_name) {
            Some(source) => {
                unsafe {
                    libobs::obs_source_filter_remove(source.0.source.0, filter_ref.source.0);
                }
                Ok(())
            }
            _ => Err(ObsError::SourceNotFound),
        }
    }

    pub fn get_source_position(&self, name: &str) -> Result<Vec2, ObsError> {
        match self.sources
                .borrow()
                .iter()
                .find(|x| x.0.name() == name) {
            Some(x) => {
                unsafe {
                    let mut main_pos: libobs::vec2 = std::mem::zeroed();
                    libobs::obs_sceneitem_get_pos(x.1.0, &mut main_pos);
                    Ok(Vec2::from(main_pos))
                }

            }
            None => Err(ObsError::SourceNotFound),
        }
    }

    pub fn get_source_scale(&self, name: &str) -> Result<Vec2, ObsError> {
        match self.sources
                .borrow()
                .iter()
                .find(|x| x.0.name() == name) {
            Some(x) => {
                unsafe {
                    let mut main_pos: libobs::vec2 = std::mem::zeroed();
                    libobs::obs_sceneitem_get_scale(x.1.0, &mut main_pos);
                    Ok(Vec2::from(main_pos))
                }

            }
            None => Err(ObsError::SourceNotFound),
        }
    }

    pub fn set_source_position(&self, name: &str, position: Vec2) -> Result<(), ObsError> {
        match self.sources
                .borrow()
                .iter()
                .find(|x| x.0.name() == name) {
            Some(x) => {
                unsafe {
                    libobs::obs_sceneitem_set_pos(x.1.0, &position.into());
                    Ok(())
                }
            }
            None => Err(ObsError::SourceNotFound),
        }
    }

    pub fn set_source_scale(&self, name: &str, scale: Vec2) -> Result<(), ObsError> {
        match self.sources
                .borrow()
                .iter()
                .find(|x| x.0.name() == name) {
            Some(x) => {
                unsafe {
                    libobs::obs_sceneitem_set_pos(x.1.0, &scale.into());
                    Ok(())
                }
            }
            None => Err(ObsError::SourceNotFound),
        }
    }

    pub fn custom(&self) {
        unsafe {
            let scene_item =
                libobs::obs_scene_find_source(self.scene.0, ObsString::new("CAMERA").as_ptr());
            let mut main_pos: libobs::vec2 = std::mem::zeroed();
            let mut main_scale: libobs::vec2 = std::mem::zeroed();
            let obs_bound_type = libobs::obs_sceneitem_get_bounds_type(scene_item);

            libobs::obs_sceneitem_get_pos(scene_item, &mut main_pos);
            libobs::obs_sceneitem_get_scale(scene_item, &mut main_scale);

            let x = main_pos.__bindgen_anon_1.__bindgen_anon_1.x
                + (main_scale.__bindgen_anon_1.__bindgen_anon_1.x * 0.8);
            let y = main_pos.__bindgen_anon_1.__bindgen_anon_1.y
                + (main_scale.__bindgen_anon_1.__bindgen_anon_1.y * 0.8);
            let camera_pos = create_vec2(x, y);
            let camera_scale = create_vec2(0.2, 0.2);

            libobs::obs_sceneitem_set_pos(scene_item, &camera_pos);
            libobs::obs_sceneitem_set_scale(scene_item, &camera_scale);

            let crop_filter = libobs::obs_source_create(
                ObsString::new("mask_filter_v2").as_ptr(),
                ObsString::new("Image Mask/Blend").as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            // Set crop settings to make it circular
            let mut mask_settings = libobs::obs_data_create();
            // libobs::obs_data_set_int(crop_settings, ObsString::new("left").as_ptr(), 0);
            // libobs::obs_data_set_int(crop_settings, ObsString::new("top").as_ptr(), 0);
            // libobs::obs_data_set_int(crop_settings, ObsString::new("right").as_ptr(), 0);
            // libobs::obs_data_set_int(crop_settings, ObsString::new("bottom").as_ptr(), 0);

            // let mut mask_settings = obs_data_create();
            // libobs::obs_data_set_string(mask_settings, ObsString::new("type").as_ptr(), ObsString::new("image_mask").as_ptr());
            libobs::obs_data_set_string(
                mask_settings,
                ObsString::new("image_path").as_ptr(),
                ObsPath::from_relative("obs-plugins/64bit/circular_mask.jpg")
                    .build()
                    .as_ptr(),
            );
            // libobs::obs_data_set_int(mask_settings, ObsString::new("opacity").as_ptr(), 100);
            // libobs::obs_data_set_bool(mask_settings, ObsString::new("invert").as_ptr(), false);
            libobs::obs_source_update(crop_filter, mask_settings);
            libobs::obs_data_release(mask_settings);

            // Add the filter to the camera source
            let source = self.get_source_mut("CAMERA").unwrap();
            libobs::obs_source_filter_add(source.source.0, crop_filter);
        }
    }

    pub fn as_ptr(&self) -> *mut obs_scene_t {
        self.scene.0
    }
}

fn create_vec2(x: f32, y: f32) -> libobs::vec2 {
    libobs::vec2 {
        __bindgen_anon_1: libobs::vec2__bindgen_ty_1 {
            __bindgen_anon_1: libobs::vec2__bindgen_ty_1__bindgen_ty_1 { x, y },
        },
    }
}
