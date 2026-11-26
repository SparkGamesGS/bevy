use crate::renderer::{wgpu_wrapper, RenderDevice, RenderQueue};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    resource::Resource,
    world::{FromWorld, World},
};
use bevy_image::ImageSamplerDescriptor;
use bevy_utils::define_atomic_id;
use core::ops::Deref;

define_atomic_id!(TextureId);

wgpu_wrapper! {
    #[derive(Clone, Debug)]
    struct WgpuTexture(wgpu::Texture);
}

/// A GPU-accessible texture.
///
/// May be converted from and dereferences to a wgpu [`Texture`](wgpu::Texture).
/// Can be created via [`RenderDevice::create_texture`](crate::renderer::RenderDevice::create_texture).
///
/// Other options for storing GPU-accessible data are:
/// * [`BufferVec`](crate::render_resource::BufferVec)
/// * [`DynamicStorageBuffer`](crate::render_resource::DynamicStorageBuffer)
/// * [`DynamicUniformBuffer`](crate::render_resource::DynamicUniformBuffer)
/// * [`GpuArrayBuffer`](crate::render_resource::GpuArrayBuffer)
/// * [`RawBufferVec`](crate::render_resource::RawBufferVec)
/// * [`StorageBuffer`](crate::render_resource::StorageBuffer)
/// * [`UniformBuffer`](crate::render_resource::UniformBuffer)
#[derive(Clone, Debug)]
pub struct Texture {
    id: TextureId,
    value: WgpuTexture,
}

impl Texture {
    /// Returns the [`TextureId`].
    #[inline]
    pub fn id(&self) -> TextureId {
        self.id
    }

    /// Creates a view of this texture.
    pub fn create_view(&self, desc: &wgpu::TextureViewDescriptor) -> TextureView {
        TextureView::from(self.value.create_view(desc))
    }
}

impl From<wgpu::Texture> for Texture {
    fn from(value: wgpu::Texture) -> Self {
        Texture {
            id: TextureId::new(),
            value: WgpuTexture::new(value),
        }
    }
}

impl Deref for Texture {
    type Target = wgpu::Texture;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// The number of array layers to give a 2D texture of `size` that will be
/// viewed as `view_dimension` on `backend`: `size.depth_or_array_layers`, or
/// more where the backend needs it.
///
/// On OpenGL a texture's kind (`TEXTURE_2D`, `TEXTURE_2D_ARRAY`,
/// `TEXTURE_CUBE_MAP` or `TEXTURE_CUBE_MAP_ARRAY`) is fixed when it is made,
/// so wgpu's OpenGL backend picks it from the size alone: one layer makes a 2D
/// texture, six layers of a square texture a cube map, any other multiple of
/// six a cube map array, and anything else a 2D array. A view of another
/// dimension then binds a texture the shader's sampler doesn't see, and wgpu
/// logs "wgpu-hal heuristics assumed that the view dimension will be equal
/// to ..." each time it is bound. So on OpenGL a texture viewed as a
/// [`D2Array`](wgpu::TextureViewDimension::D2Array) gets at least two layers
/// and never a cube's count, and one viewed as a
/// [`CubeArray`](wgpu::TextureViewDimension::CubeArray) at least two cubes.
/// Every other backend keeps the count it is given.
pub fn array_layers_for_view(
    backend: wgpu::Backend,
    view_dimension: wgpu::TextureViewDimension,
    size: wgpu::Extent3d,
) -> u32 {
    let layers = size.depth_or_array_layers;
    if backend != wgpu::Backend::Gl {
        return layers;
    }
    match view_dimension {
        wgpu::TextureViewDimension::D2Array => {
            let layers = layers.max(2);
            if size.width == size.height && layers.is_multiple_of(6) {
                layers + 1
            } else {
                layers
            }
        }
        wgpu::TextureViewDimension::CubeArray => layers.div_ceil(6).max(2) * 6,
        _ => layers,
    }
}

define_atomic_id!(TextureViewId);

wgpu_wrapper! {
    #[derive(Clone, Debug)]
    struct WgpuTextureView(wgpu::TextureView);

    struct WgpuSurfaceTexture(wgpu::SurfaceTexture);
}

/// Describes a [`Texture`] with its associated metadata required by a pipeline or [`BindGroup`](super::BindGroup).
#[derive(Clone, Debug)]
pub struct TextureView {
    id: TextureViewId,
    value: WgpuTextureView,
}

pub struct SurfaceTexture {
    value: WgpuSurfaceTexture,
}

impl SurfaceTexture {
    pub fn present(self, render_queue: &RenderQueue) {
        render_queue.present(self.value.into_inner());
    }
}

impl TextureView {
    /// Returns the [`TextureViewId`].
    #[inline]
    pub fn id(&self) -> TextureViewId {
        self.id
    }
}

impl From<wgpu::TextureView> for TextureView {
    fn from(value: wgpu::TextureView) -> Self {
        TextureView {
            id: TextureViewId::new(),
            value: WgpuTextureView::new(value),
        }
    }
}

impl From<wgpu::SurfaceTexture> for SurfaceTexture {
    fn from(value: wgpu::SurfaceTexture) -> Self {
        SurfaceTexture {
            value: WgpuSurfaceTexture::new(value),
        }
    }
}

impl Deref for TextureView {
    type Target = wgpu::TextureView;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Deref for SurfaceTexture {
    type Target = wgpu::SurfaceTexture;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

define_atomic_id!(SamplerId);

wgpu_wrapper! {
    #[derive(Clone, Debug)]
    struct WgpuSampler(wgpu::Sampler);
}

/// A Sampler defines how a pipeline will sample from a [`TextureView`].
/// They define image filters (including anisotropy) and address (wrapping) modes, among other things.
///
/// May be converted from and dereferences to a wgpu [`Sampler`](wgpu::Sampler).
/// Can be created via [`RenderDevice::create_sampler`](crate::renderer::RenderDevice::create_sampler).
#[derive(Clone, Debug)]
pub struct Sampler {
    id: SamplerId,
    value: WgpuSampler,
}

impl Sampler {
    /// Returns the [`SamplerId`].
    #[inline]
    pub fn id(&self) -> SamplerId {
        self.id
    }
}

impl From<wgpu::Sampler> for Sampler {
    fn from(value: wgpu::Sampler) -> Self {
        Sampler {
            id: SamplerId::new(),
            value: WgpuSampler::new(value),
        }
    }
}

impl Deref for Sampler {
    type Target = wgpu::Sampler;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Stores the [`ImageSamplerDescriptor`] used to create the [`DefaultImageSampler`].
///
/// This is kept as a resource so that [`DefaultImageSampler`] can be recreated on GPU device recovery.
#[derive(Resource, Debug, Clone, Deref)]
pub struct DefaultImageSamplerDescriptor(pub ImageSamplerDescriptor);

/// A rendering resource for the default image sampler which is set during renderer
/// initialization.
///
/// The [`ImagePlugin`](bevy_image::ImagePlugin) can be set during app initialization to change the default
/// image sampler.
#[derive(Resource, Debug, Clone, Deref, DerefMut)]
pub struct DefaultImageSampler(pub(crate) Sampler);

impl FromWorld for DefaultImageSampler {
    fn from_world(world: &mut World) -> Self {
        let descriptor = world.resource::<DefaultImageSamplerDescriptor>();
        let wgpu_descriptor = descriptor.as_wgpu();
        let device = world.resource::<RenderDevice>();
        let sampler = device.create_sampler(&wgpu_descriptor);
        Self(sampler)
    }
}

#[cfg(test)]
mod tests {
    use super::array_layers_for_view;
    use wgpu::{
        Backend, Extent3d,
        TextureViewDimension::{self, *},
    };

    /// The kind of texture wgpu's OpenGL backend makes for a 2D texture of
    /// `size` (`wgpu_hal::gles::Texture::get_info_from_desc`), as the view
    /// dimension that binds it.
    fn opengl_texture_kind(size: Extent3d) -> TextureViewDimension {
        let layers = size.depth_or_array_layers;
        match (
            size.width == size.height && layers.is_multiple_of(6),
            layers,
        ) {
            (false, 1) => D2,
            (false, _) => D2Array,
            (true, 6) => Cube,
            (true, _) => CubeArray,
        }
    }

    fn size(width: u32, layers: u32) -> Extent3d {
        Extent3d {
            width,
            height: 2048,
            depth_or_array_layers: layers,
        }
    }

    #[test]
    fn opengl_makes_the_kind_of_texture_each_view_binds() {
        for (view_dimension, width) in [
            (D2Array, 2048),
            (D2Array, 1024),
            (CubeArray, 2048),
            (D2, 2048),
            (Cube, 2048),
        ] {
            let needed: &[u32] = match view_dimension {
                D2 => &[1],
                Cube => &[6],
                _ => &[1, 2, 3, 5, 6, 7, 11, 12, 13, 18, 24, 36, 256],
            };
            for &layers in needed {
                let given = array_layers_for_view(Backend::Gl, view_dimension, size(width, layers));
                assert!(
                    given >= layers,
                    "{view_dimension:?}: {given} < {layers} layers"
                );
                assert_eq!(
                    opengl_texture_kind(size(width, given)),
                    view_dimension,
                    "{view_dimension:?} of {layers} layers ({width}x2048) got {given}",
                );
            }
        }
    }

    #[test]
    fn opengl_adds_layers_only_where_needed() {
        let gl = |view_dimension, layers| {
            array_layers_for_view(Backend::Gl, view_dimension, size(2048, layers))
        };
        assert_eq!(gl(D2Array, 1), 2);
        assert_eq!(gl(D2Array, 4), 4);
        assert_eq!(gl(D2Array, 6), 7);
        assert_eq!(gl(CubeArray, 6), 12);
        assert_eq!(gl(CubeArray, 18), 18);
        assert_eq!(gl(D2, 1), 1);
        assert_eq!(gl(Cube, 6), 6);
    }

    #[test]
    fn other_backends_keep_the_layer_count() {
        for backend in [
            Backend::Vulkan,
            Backend::Metal,
            Backend::Dx12,
            Backend::BrowserWebGpu,
            Backend::Noop,
        ] {
            for (view_dimension, layers) in [(D2Array, 1), (D2Array, 6), (CubeArray, 6), (D2, 1)] {
                assert_eq!(
                    array_layers_for_view(backend, view_dimension, size(2048, layers)),
                    layers
                );
            }
        }
    }
}
