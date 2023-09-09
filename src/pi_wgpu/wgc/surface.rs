use super::super::{hal, wgt, Adapter, Device, SurfaceCapabilities, Texture, TextureFormat};

/// Handle to a presentable surface.
///
/// A `Surface` represents a platform-specific surface (e.g. a window) onto which rendered images may
/// be presented. A `Surface` may be created with the unsafe function [`Instance::create_surface`].
///
/// This type is unique to the Rust API of `wgpu`. In the WebGPU specification,
/// [`GPUCanvasContext`](https://gpuweb.github.io/gpuweb/#canvas-context)
/// serves a similar role.
#[derive(Debug)]
pub struct Surface {
    pub(crate) inner: hal::Surface,
}

impl Surface {
    /// Returns the capabilities of the surface when used with the given adapter.
    ///
    /// Returns specified values (see [`SurfaceCapabilities`]) if surface is incompatible with the adapter.
    pub fn get_capabilities(&self, adapter: &Adapter) -> SurfaceCapabilities {
        profiling::scope!("Surface::get_capabilities");

        let mut hal_caps = adapter.inner.surface_capabilities(&self.inner).unwrap();

        hal_caps.formats.sort_by_key(|f| !f.is_srgb());

        SurfaceCapabilities {
            formats: hal_caps.formats,
            present_modes: hal_caps.present_modes,
            alpha_modes: hal_caps.composite_alpha_modes,
        }
    }

    /// Return a default `SurfaceConfiguration` from width and height to use for the [`Surface`] with this adapter.
    ///
    /// Returns None if the surface isn't supported by this adapter
    pub fn get_default_config(
        &self,
        adapter: &Adapter,
        width: u32,
        height: u32,
    ) -> Option<SurfaceConfiguration> {
        let caps = self.get_capabilities(adapter);

        Some(SurfaceConfiguration {
            usage: wgt::TextureUsages::RENDER_ATTACHMENT,
            format: *caps.formats.get(0)?,
            width,
            height,
            present_mode: *caps.present_modes.get(0)?,
            alpha_mode: wgt::CompositeAlphaMode::Auto,
            view_formats: vec![],
        })
    }

    /// Initializes [`Surface`] for presentation.
    ///
    /// # Panics
    ///
    /// - A old [`SurfaceTexture`] is still alive referencing an old surface.
    /// - Texture format requested is unsupported on the surface.
    #[inline]
    pub fn configure(&self, device: &Device, config: &SurfaceConfiguration) {
        self.inner.configure(&device.inner, config).unwrap();
    }

    /// Returns the next texture to be presented by the swapchain for drawing.
    ///
    /// In order to present the [`SurfaceTexture`] returned by this method,
    /// first a [`Queue::submit`] needs to be done with some work rendering to this texture.
    /// Then [`SurfaceTexture::present`] needs to be called.
    ///
    /// If a SurfaceTexture referencing this surface is alive when the swapchain is recreated,
    /// recreating the swapchain will panic.
    #[inline]
    pub fn get_current_texture(&self) -> Result<SurfaceTexture, SurfaceError> {
        match self.inner.acquire_texture() {
            None => Err(SurfaceError::Lost),
            Some(inner) => {
                let texture = crate::Texture::into_surface_texture(inner);
                Ok(SurfaceTexture {
                    texture,
                    suboptimal: true,
                    presented: true,
                })
            }
        }
    }
}

/// Surface texture that can be rendered to.
/// Result of a successful call to [`Surface::get_current_texture`].
///
/// This type is unique to the Rust API of `wgpu`. In the WebGPU specification,
/// the [`GPUCanvasContext`](https://gpuweb.github.io/gpuweb/#canvas-context) provides
/// a texture without any additional information.
#[derive(Debug)]
pub struct SurfaceTexture {
    /// Accessible view of the frame.
    pub texture: Texture,
    /// `true` if the acquired buffer can still be used for rendering,
    /// but should be recreated for maximum performance.
    pub suboptimal: bool,

    pub(crate) presented: bool,
}

impl SurfaceTexture {
    /// Schedule this texture to be presented on the owning surface.
    ///
    /// Needs to be called after any work on the texture is scheduled via [`Queue::submit`].
    pub fn present(mut self) {
        todo!()
    }
}

/// Result of an unsuccessful call to [`Surface::get_current_texture`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SurfaceError {
    /// A timeout was encountered while trying to acquire the next frame.
    Timeout,
    /// The underlying surface has changed, and therefore the swap chain must be updated.
    Outdated,
    /// The swap chain has been lost and needs to be recreated.
    Lost,
    /// There is no more memory left to allocate a new frame.
    OutOfMemory,
}

impl std::fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Timeout => "A timeout was encountered while trying to acquire the next frame",
            Self::Outdated => "The underlying surface has changed, and therefore the swap chain must be updated",
            Self::Lost =>  "The swap chain has been lost and needs to be recreated",
            Self::OutOfMemory => "There is no more memory left to allocate a new frame",
        })
    }
}

impl std::error::Error for SurfaceError {}

/// [`Instance::create_surface()`] or a related function failed.
#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct CreateSurfaceError {}

impl std::fmt::Display for CreateSurfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Creating a surface failed")
    }
}

impl std::error::Error for CreateSurfaceError {}

/// Describes a [`Surface`].
///
/// For use with [`Surface::configure`].
///
/// Corresponds to [WebGPU `GPUCanvasConfiguration`](
/// https://gpuweb.github.io/gpuweb/#canvas-configuration).
pub type SurfaceConfiguration = wgt::SurfaceConfiguration<Vec<TextureFormat>>;
