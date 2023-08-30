use std::num::NonZeroU32;

use crate::{
    hal, Extent3d, Label, TextureAspect, TextureDimension, TextureFormat, TextureUsages,
    TextureViewDimension,
};

/// Handle to a texture on the GPU.
///
/// It can be created with [`Device::create_texture`].
///
/// Corresponds to [WebGPU `GPUTexture`](https://gpuweb.github.io/gpuweb/#texture-interface).
#[derive(Debug)]
pub struct Texture {
    pub(crate) inner: hal::Texture,

    pub size: Extent3d,
    /// Mip count of texture. For a texture with no extra mips, this must be 1.
    pub mip_level_count: u32,
    /// Sample count of texture. If this is not 1, texture must have [`BindingType::Texture::multisampled`] set to true.
    pub sample_count: u32,
    /// Dimensions of the texture.
    pub dimension: TextureDimension,
    /// Format of the texture.
    pub format: TextureFormat,
    /// Allowed usages of the texture. If used in other ways, the operation will panic.
    pub usage: TextureUsages,
    /// Specifies what view formats will be allowed when calling create_view() on this texture.
    ///
    /// View formats of the same format as the texture are always allowed.
    ///
    /// Note: currently, only the srgb-ness is allowed to change. (ex: Rgba8Unorm texture + Rgba8UnormSrgb view)
    pub view_formats: V,
}

impl Texture {
    #[inline]
    pub(crate) fn from_hal(inner: crate::hal::Texture, desc: &crate::TextureDescriptor) -> Self {
        Self { inner }
    }
}

impl Texture {
    /// Creates a view of this texture.
    #[inline]
    pub fn create_view(&self, desc: &TextureViewDescriptor) -> TextureView {
        let inner = hal::TextureView::new(&self.inner, desc).unwrap();
        TextureView { inner }
    }

    /// Make an `ImageCopyTexture` representing the whole texture.
    #[inline]
    pub fn as_image_copy(&self) -> ImageCopyTexture {
        unimplemented!("Texture::as_image_copy is not implemented")
    }

    /// Returns the size of this `Texture`.
    ///
    /// This is always equal to the `size` that was specified when creating the texture.
    #[inline]
    pub fn size(&self) -> Extent3d {
        self.inner.copy_size
    }

    /// Returns the width of this `Texture`.
    ///
    /// This is always equal to the `size.width` that was specified when creating the texture.
    #[inline]
    pub fn width(&self) -> u32 {
        todo!("Texture::width is not implemented")
    }

    /// Returns the height of this `Texture`.
    ///
    /// This is always equal to the `size.height` that was specified when creating the texture.
    #[inline]
    pub fn height(&self) -> u32 {
        unimplemented!("Texture::height is not implemented")
    }

    /// Returns the depth or layer count of this `Texture`.
    ///
    /// This is always equal to the `size.depth_or_array_layers` that was specified when creating the texture.
    #[inline]
    pub fn depth_or_array_layers(&self) -> u32 {
        unimplemented!("Texture::depth_or_array_layers is not implemented")
    }

    /// Returns the mip_level_count of this `Texture`.
    ///
    /// This is always equal to the `mip_level_count` that was specified when creating the texture.
    #[inline]
    pub fn mip_level_count(&self) -> u32 {
        unimplemented!("Texture::mip_level_count is not implemented")
    }

    /// Returns the sample_count of this `Texture`.
    ///
    /// This is always equal to the `sample_count` that was specified when creating the texture.
    #[inline]
    pub fn sample_count(&self) -> u32 {
        unimplemented!("Texture::sample_count is not implemented")
    }

    /// Returns the dimension of this `Texture`.
    ///
    /// This is always equal to the `dimension` that was specified when creating the texture.
    #[inline]
    pub fn dimension(&self) -> TextureDimension {
        unimplemented!("Texture::dimension is not implemented")
    }

    /// Returns the format of this `Texture`.
    ///
    /// This is always equal to the `format` that was specified when creating the texture.
    #[inline]
    pub fn format(&self) -> TextureFormat {
        unimplemented!("Texture::format is not implemented")
    }

    /// Returns the allowed usages of this `Texture`.
    ///
    /// This is always equal to the `usage` that was specified when creating the texture.
    #[inline]
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
pub struct TextureView {
    pub(crate) inner: hal::TextureView,
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

/// Describes a [`Texture`].
///
/// For use with [`Device::create_texture`].
///
/// Corresponds to [WebGPU `GPUTextureDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gputexturedescriptor).
pub type TextureDescriptor<'a> = crate::wgt::TextureDescriptor<Label<'a>, &'a [TextureFormat]>;

pub use crate::wgt::ImageCopyTexture as ImageCopyTextureBase;

/// View of a texture which can be used to copy to/from a buffer/texture.
///
/// Corresponds to [WebGPU `GPUImageCopyTexture`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpuimagecopytexture).
pub type ImageCopyTexture<'a> = ImageCopyTextureBase<&'a Texture>;
