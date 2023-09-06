use std::num::NonZeroU32;

use super::super::{
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

    size_impl: Extent3d,
    mip_level_count_impl: u32,
    sample_count_impl: u32,
    dimension_impl: TextureDimension,
    format_impl: TextureFormat,
    usage_impl: TextureUsages,
}

impl Texture {
    #[inline]
    pub(crate) fn from_hal(inner: super::super::hal::Texture, desc: &super::super::TextureDescriptor) -> Self {
        Self {
            inner,
            size_impl: desc.size,
            mip_level_count_impl: desc.mip_level_count,
            sample_count_impl: desc.sample_count,
            dimension_impl: desc.dimension,
            format_impl: desc.format,
            usage_impl: desc.usage,
        }
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
        todo!("")
    }

    /// Returns the size of this `Texture`.
    ///
    /// This is always equal to the `size` that was specified when creating the texture.
    #[inline]
    pub fn size(&self) -> Extent3d {
        self.size_impl
    }

    /// Returns the width of this `Texture`.
    ///
    /// This is always equal to the `size.width` that was specified when creating the texture.
    #[inline]
    pub fn width(&self) -> u32 {
        self.size_impl.width
    }

    /// Returns the height of this `Texture`.
    ///
    /// This is always equal to the `size.height` that was specified when creating the texture.
    #[inline]
    pub fn height(&self) -> u32 {
        self.size_impl.height
    }

    /// Returns the depth or layer count of this `Texture`.
    ///
    /// This is always equal to the `size.depth_or_array_layers` that was specified when creating the texture.
    #[inline]
    pub fn depth_or_array_layers(&self) -> u32 {
        self.size_impl.depth_or_array_layers
    }

    /// Returns the mip_level_count of this `Texture`.
    ///
    /// This is always equal to the `mip_level_count` that was specified when creating the texture.
    #[inline]
    pub fn mip_level_count(&self) -> u32 {
        self.mip_level_count_impl
    }

    /// Returns the sample_count of this `Texture`.
    ///
    /// This is always equal to the `sample_count` that was specified when creating the texture.
    #[inline]
    pub fn sample_count(&self) -> u32 {
        self.sample_count_impl
    }

    /// Returns the dimension of this `Texture`.
    ///
    /// This is always equal to the `dimension` that was specified when creating the texture.
    #[inline]
    pub fn dimension(&self) -> TextureDimension {
        self.dimension_impl
    }

    /// Returns the format of this `Texture`.
    ///
    /// This is always equal to the `format` that was specified when creating the texture.
    #[inline]
    pub fn format(&self) -> TextureFormat {
        self.format_impl
    }

    /// Returns the allowed usages of this `Texture`.
    ///
    /// This is always equal to the `usage` that was specified when creating the texture.
    #[inline]
    pub fn usage(&self) -> TextureUsages {
        self.usage_impl
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

impl TextureView {
    // 返回 (w, h, d)
    #[inline]
    pub(crate) fn get_size(&self) -> (u32, u32, u32) {
        let size  = &self.inner.inner.copy_size;
        (size.width, size.height, size.depth)
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
    pub mip_level_count: Option<u32>,
    /// Base array layer.
    pub base_array_layer: u32,
    /// Layer count.
    /// If `Some(count)`, `base_array_layer + count` must be less or equal to the underlying array count.
    /// If `None`, considered to include the rest of the array layers, but at least 1 in total.
    pub array_layer_count: Option<u32>,
}

/// Describes a [`Texture`].
///
/// For use with [`Device::create_texture`].
///
/// Corresponds to [WebGPU `GPUTextureDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gputexturedescriptor).
pub type TextureDescriptor<'a> = super::super::wgt::TextureDescriptor<Label<'a>, &'a [TextureFormat]>;

pub use super::super::wgt::ImageCopyTexture as ImageCopyTextureBase;

/// View of a texture which can be used to copy to/from a buffer/texture.
///
/// Corresponds to [WebGPU `GPUImageCopyTexture`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpuimagecopytexture).
pub type ImageCopyTexture<'a> = ImageCopyTextureBase<&'a Texture>;
