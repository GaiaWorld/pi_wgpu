use super::api::HalApi;
use crate::{
    wgpu_hal as hal, Extent3d, Label, TextureAspect, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDimension,
};
use std::num::NonZeroU32;

/// Handle to a texture on the GPU.
///
/// It can be created with [`Device::create_texture`].
///
/// Corresponds to [WebGPU `GPUTexture`](https://gpuweb.github.io/gpuweb/#texture-interface).
#[derive(Debug)]
pub struct Texture {
    inner: <hal::GL as hal::Api>::Texture,
}
static_assertions::assert_impl_all!(Texture: Send, Sync);

impl Drop for Texture {
    fn drop(&mut self) {
        unimplemented!("Texture::drop")
    }
}

impl Texture {
    /// Returns the inner hal Texture using a callback. The hal texture will be `None` if the
    /// backend type argument does not match with this wgpu Texture
    ///
    /// # Safety
    ///
    /// - The raw handle obtained from the hal Texture must not be manually destroyed
    #[cfg(any(not(target_arch = "wasm32"), feature = "emscripten"))]
    pub unsafe fn as_hal<A: HalApi, F: FnOnce(Option<&A::Texture>)>(
        &self,
        hal_texture_callback: F,
    ) {
        unimplemented!("Texture::as_hal is not implemented")
    }

    /// Creates a view of this texture.
    pub fn create_view(&self, desc: &TextureViewDescriptor) -> TextureView {
        unimplemented!("Texture::create_view is not implemented")
    }

    /// Destroy the associated native resources as soon as possible.
    pub fn destroy(&self) {
        unimplemented!("Texture::destroy is not implemented")
    }

    /// Make an `ImageCopyTexture` representing the whole texture.
    pub fn as_image_copy(&self) -> ImageCopyTexture {
        unimplemented!("Texture::as_image_copy is not implemented")
    }

    /// Returns the size of this `Texture`.
    ///
    /// This is always equal to the `size` that was specified when creating the texture.
    pub fn size(&self) -> Extent3d {
        unimplemented!("Texture::size is not implemented")
    }

    /// Returns the width of this `Texture`.
    ///
    /// This is always equal to the `size.width` that was specified when creating the texture.
    pub fn width(&self) -> u32 {
        unimplemented!("Texture::width is not implemented")
    }

    /// Returns the height of this `Texture`.
    ///
    /// This is always equal to the `size.height` that was specified when creating the texture.
    pub fn height(&self) -> u32 {
        unimplemented!("Texture::height is not implemented")
    }

    /// Returns the depth or layer count of this `Texture`.
    ///
    /// This is always equal to the `size.depth_or_array_layers` that was specified when creating the texture.
    pub fn depth_or_array_layers(&self) -> u32 {
        unimplemented!("Texture::depth_or_array_layers is not implemented")
    }

    /// Returns the mip_level_count of this `Texture`.
    ///
    /// This is always equal to the `mip_level_count` that was specified when creating the texture.
    pub fn mip_level_count(&self) -> u32 {
        unimplemented!("Texture::mip_level_count is not implemented")
    }

    /// Returns the sample_count of this `Texture`.
    ///
    /// This is always equal to the `sample_count` that was specified when creating the texture.
    pub fn sample_count(&self) -> u32 {
        unimplemented!("Texture::sample_count is not implemented")
    }

    /// Returns the dimension of this `Texture`.
    ///
    /// This is always equal to the `dimension` that was specified when creating the texture.
    pub fn dimension(&self) -> TextureDimension {
        unimplemented!("Texture::dimension is not implemented")
    }

    /// Returns the format of this `Texture`.
    ///
    /// This is always equal to the `format` that was specified when creating the texture.
    pub fn format(&self) -> TextureFormat {
        unimplemented!("Texture::format is not implemented")
    }

    /// Returns the allowed usages of this `Texture`.
    ///
    /// This is always equal to the `usage` that was specified when creating the texture.
    pub fn usage(&self) -> TextureUsages {
        unimplemented!("Texture::usage is not implemented")
    }
}

/// Handle to a texture view.
///
/// A `TextureView` object describes a texture and associated metadata needed by a
/// [`RenderPipeline`] or [`BindGroup`].
///
/// Corresponds to [WebGPU `GPUTextureView`](https://gpuweb.github.io/gpuweb/#gputextureview).
#[derive(Debug)]
pub struct TextureView {}
static_assertions::assert_impl_all!(TextureView: Send, Sync);

impl Drop for TextureView {
    fn drop(&mut self) {
        unimplemented!("TextureView::drop is not implemented")
    }
}

/// Describes a [`TextureView`].
///
/// For use with [`Texture::create_view`].
///
/// Corresponds to [WebGPU `GPUTextureViewDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gputextureviewdescriptor).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextureViewDescriptor<'a> {
    /// Debug label of the texture view. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
    /// Format of the texture view. At this time, it must be the same as the underlying format of the texture.
    pub format: Option<TextureFormat>,
    /// The dimension of the texture view. For 1D textures, this must be `D1`. For 2D textures it must be one of
    /// `D2`, `D2Array`, `Cube`, and `CubeArray`. For 3D textures it must be `D3`
    pub dimension: Option<TextureViewDimension>,
    /// Aspect of the texture. Color textures must be [`TextureAspect::All`].
    pub aspect: TextureAspect,
    /// Base mip level.
    pub base_mip_level: u32,
    /// Mip level count.
    /// If `Some(count)`, `base_mip_level + count` must be less or equal to underlying texture mip count.
    /// If `None`, considered to include the rest of the mipmap levels, but at least 1 in total.
    pub mip_level_count: Option<NonZeroU32>,
    /// Base array layer.
    pub base_array_layer: u32,
    /// Layer count.
    /// If `Some(count)`, `base_array_layer + count` must be less or equal to the underlying array count.
    /// If `None`, considered to include the rest of the array layers, but at least 1 in total.
    pub array_layer_count: Option<NonZeroU32>,
}
static_assertions::assert_impl_all!(TextureViewDescriptor: Send, Sync);

/// Describes a [`Texture`].
///
/// For use with [`Device::create_texture`].
///
/// Corresponds to [WebGPU `GPUTextureDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gputexturedescriptor).
pub type TextureDescriptor<'a> = wgt::TextureDescriptor<Label<'a>, &'a [TextureFormat]>;
static_assertions::assert_impl_all!(TextureDescriptor: Send, Sync);

pub use wgt::ImageCopyTexture as ImageCopyTextureBase;

/// View of a texture which can be used to copy to/from a buffer/texture.
///
/// Corresponds to [WebGPU `GPUImageCopyTexture`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpuimagecopytexture).
pub type ImageCopyTexture<'a> = ImageCopyTextureBase<&'a Texture>;
static_assertions::assert_impl_all!(ImageCopyTexture: Send, Sync);
