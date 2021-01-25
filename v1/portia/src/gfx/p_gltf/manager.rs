use crate::{
    file_system::{Asset, AssetState, FileSystem, LoadableAsset},
    gfx::DeviceQueue,
    input::Input,
    time::Clock,
    EventQueue,
};

use super::scene;

/// GLTF scene manager.
pub struct GltfManager {
    scenes: Vec<(&'static str, GltfScene)>,
}

impl GltfManager {
    /// Creates a new GLTF manager
    pub fn new(max_scenes: usize) -> Self {
        Self {
            scenes: Vec::with_capacity(max_scenes),
        }
    }

    fn find_index(&self, file: &'static str) -> Option<usize> {
        for (i, (name, _)) in self.scenes.iter().enumerate() {
            if *name == file {
                return Some(i);
            }
        }

        None
    }

    /// Attempts to load the scene in the background
    pub fn load_scene(&mut self, file: &'static str) {
        match self.find_index(file) {
            Some(_) => {}
            None => {
                self.scenes.push((file, GltfScene::load(file)));
            }
        }
    }

    /// Drops the given scene
    pub fn drop_scene(&mut self, file: &'static str) {
        match self.find_index(file) {
            Some(i) => {
                self.scenes.remove(i);
            }
            None => {}
        }
    }

    /// Goes through all scenes, uploading them to the GPU if they're ready.
    pub fn process(
        &mut self,
        model_layout: &wgpu::BindGroupLayout,
        node_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        dq: &DeviceQueue,
        event_queue: &mut EventQueue,
    ) {
        for (_name, scene) in self.scenes.iter_mut() {
            scene.process(model_layout, node_layout, material_layout, dq, event_queue);
        }
    }

    pub fn render<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
        for (_, scene) in &self.scenes {
            scene.render(render_pass);
        }
    }
}

/// A intermediate representation of what the file system will return
pub struct LoadableGltf {
    pub document: gltf::Document,
    pub buffers: std::vec::Vec<gltf::buffer::Data>,
    pub images: std::vec::Vec<gltf::image::Data>,
}

impl LoadableAsset for LoadableGltf {
    fn asset_from_file(file: &'static str) -> Self {
        let path = FileSystem::res_dir().join(file);
        let (document, buffers, images) = gltf::import(path).unwrap();

        return Self {
            document,
            buffers,
            images,
        };
    }
}

/// An interactable GLTF scene. Loads in the background.
struct GltfScene {
    asset: Asset<LoadableGltf>,
    scene: Option<scene::Scene>,
}

impl GltfScene {
    pub fn load(file: &'static str) -> Self {
        Self {
            asset: Asset::load(file),
            scene: None,
        }
    }

    pub fn process(
        &mut self,
        model_layout: &wgpu::BindGroupLayout,
        node_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        dq: &DeviceQueue,
        event_queue: &mut EventQueue,
    ) {
        match self.asset.state() {
            AssetState::Buffering => match self.asset.try_receive() {
                AssetState::Ready => {
                    // File has been loaded and is ready to consume.
                    self.process(model_layout, node_layout, material_layout, dq, event_queue);
                }
                _ => {}
            },
            AssetState::Ready => {
                event_queue.push(Input::AssetLoaded {
                    file: self.asset.file(),
                });
                let asset_files = self.asset.consume();
                match asset_files {
                    Some(gltf_data) => {
                        let scene = scene::Scene::initialize(
                            *gltf_data,
                            model_layout,
                            node_layout,
                            material_layout,
                            dq,
                        );
                        self.scene = Some(scene);
                    }
                    None => {
                        panic!("Unable to parse GLTF file!");
                    }
                }
                // TODO: wire up to GPU
            }
            AssetState::Consumed => {
                if let Some(ref mut scene) = self.scene {
                    scene.update();
                }
            }
        }
    }

    fn render<'b>(&'b self, render_pass: &mut wgpu::RenderPass<'b>) {
        match &self.scene {
            Some(scene) => {
                scene.render(render_pass);
            }
            None => {}
        }
    }
}
