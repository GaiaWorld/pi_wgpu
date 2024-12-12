use std::{future::{ready, Future}, marker::PhantomData};

use pi_share::Share;
use raw_window_handle::{DisplayHandle, HasDisplayHandle, HasRawDisplayHandle, HasRawWindowHandle, HasWindowHandle, WindowHandle};

use crate::{pi_wgpu::wgt::Gles3MinorVersion, SurfaceTarget};

use super::super::{
    hal, wgt, Backends, CreateSurfaceError, InstanceDescriptor, PowerPreference, Surface,
};

/// Context for all other wgpu objects. Instance of wgpu.
///
/// This is the first thing you create when using wgpu.
/// Its primary use is to create [`Adapter`]s and [`Surface`]s.
///
/// Does not have to be kept alive.
///
/// Corresponds to [WebGPU `GPU`](https://gpuweb.github.io/gpuweb/#gpu-interface).
#[derive(Debug, Clone)]
pub struct Instance {
    pub(crate) inner: Share<hal::Instance>,
}

impl Default for Instance {
    /// Creates a new instance of wgpu with default options.
    ///
    /// Backends are set to `Backends::all()`, and FXC is chosen as the `dx12_shader_compiler`.
    fn default() -> Self {
        Instance::new(InstanceDescriptor {
            backends: Backends::GL,
            dx12_shader_compiler: wgt::Dx12Compiler::default(),
            flags: Default::default(),
            gles_minor_version: Gles3MinorVersion::Automatic,
        })
    }
}

impl Instance {
    /// Create an new instance of wgpu.
    ///
    /// # Arguments
    ///
    /// - `instance_desc` - Has fields for which [backends][Backends] wgpu will choose
    ///   during instantiation, and which [DX12 shader compiler][Dx12Compiler] wgpu will use.
    pub fn new(mut instance_desc: InstanceDescriptor) -> Self {
        profiling::scope!("Instance::new");

        log::trace!("pi_wgpu::Instance::new, instance_desc{:?}", instance_desc);

        // assert!(instance_desc.backends.contains(Backends::GL));
        instance_desc.backends = Backends::GL;

        let mut flags = hal::InstanceFlags::empty();
        if cfg!(debug_assertions) {
            flags |= hal::InstanceFlags::VALIDATION;

            // 小米9 手机 会崩溃
            // flags |= InstanceFlags::DEBUG;
        }

        let hal_desc = hal::InstanceDescriptor {
            name: "pi_wgpu:gl",
            flags,
            dx12_shader_compiler: instance_desc.dx12_shader_compiler.clone(),
        };

        let imp = hal::Instance::init(&hal_desc).unwrap();

        Self {
            inner: Share::new(imp),
        }
    }

    /// Retrieves an [`Adapter`] which matches the given [`RequestAdapterOptions`].
    ///
    /// Some options are "soft", so treated as non-mandatory. Others are "hard".
    ///
    /// If no adapters are found that suffice all the "hard" options, `None` is returned.
    pub fn request_adapter(
        &self,
        options: &super::super::RequestAdapterOptions,
    ) -> impl Future<Output = Option<super::super::Adapter>> + Send {
        profiling::scope!("Instance::request_adapter");

        // log::trace!("pi_wgpu::Instance::request_adapter, options = {:?}", options);

        // 不支持 软件 Adapter
        assert!(!options.force_fallback_adapter);

        let mut adapters = self.inner.enumerate_adapters();

        if let Some(surface) = options.compatible_surface {
            let surface = &surface.inner;

            adapters.retain(|exposed| exposed.adapter.surface_capabilities(&surface).is_some());
        }

        let mut device_types = Vec::new();
        device_types.extend(adapters.iter().map(|ad| ad.info.device_type));

        if device_types.is_empty() {
            log::warn!("No adapters found!");
            return ready(None);
        }

        let (mut integrated, mut discrete, mut virt, mut cpu, mut other) = (-1, -1, -1, -1, -1);

        for (i, ty) in device_types.into_iter().enumerate() {
            match ty {
                wgt::DeviceType::IntegratedGpu => {
                    if integrated < 0 {
                        integrated = i as i32;
                    }
                }
                wgt::DeviceType::DiscreteGpu => {
                    if discrete < 0 {
                        discrete = i as i32;
                    }
                }
                wgt::DeviceType::VirtualGpu => {
                    if virt < 0 {
                        virt = i as i32;
                    }
                }
                wgt::DeviceType::Cpu => {
                    if cpu < 0 {
                        cpu = i as i32;
                    }
                }
                wgt::DeviceType::Other => {
                    if other < 0 {
                        other = i as i32;
                    }
                }
            }
        }

        let select;
        match options.power_preference {
            // 低性能：集成显卡 --> 独立显卡 --> 其他 --> 虚拟显卡 --> cpu 软件模拟
            PowerPreference::LowPower => {
                select = [integrated, discrete, other, virt, cpu];
            }
            // 高性能：独立显卡 --> 集成显卡 --> 其他 --> 虚拟显卡 --> cpu 软件模拟
            PowerPreference::HighPerformance => {
                select = [discrete, integrated, other, virt, cpu];
            }
        }

        for index in select {
            if index >= 0 {
                let adapter = adapters.swap_remove(index as usize);

                let adapter = super::super::Adapter {
                    inner: adapter.adapter,
                };

                return ready(Some(adapter));
            }
        }

        return ready(None);
    }

    /// Creates a surface from a raw window handle.
    ///
    /// If the specified display and window handle are not supported by any of the backends, then the surface
    /// will not be supported by any adapters.
    ///
    /// # Safety
    ///
    /// - `raw_window_handle` must be a valid object to create a surface upon.
    /// - `raw_window_handle` must remain valid until after the returned [`Surface`] is
    ///   dropped.
    ///
    /// # Errors
    ///
    /// - On WebGL2: Will return an error if the browser does not support WebGL2,
    ///   or declines to provide GPU access (such as due to a resource shortage).
    ///
    /// # Panics
    ///
    /// - On macOS/Metal: will panic if not called on the main thread.
    /// - On web: will panic if the `raw_window_handle` does not properly refer to a
    ///   canvas element.
    pub fn create_surface<'window>(
        &self,
        window: impl Into<SurfaceTarget<'window>>,
    ) -> Result<Surface<'window>, CreateSurfaceError> {
        profiling::scope!("Instance::create_surface");
        let target: SurfaceTarget = window.into();
        let target = unsafe { std::mem::transmute(target)};
        let surface = match &target{
            SurfaceTarget::Window(window) => unsafe {
                let h = RawHandle::from_window(window).map_err(|_e| CreateSurfaceError {
                })?;
                let surface = self.inner.create_surface(&h);

                surface
            },

            #[cfg(any(webgpu, webgl))]
            SurfaceTarget::Canvas(canvas) => {
                let value: &wasm_bindgen::JsValue = &canvas;
                let obj = std::ptr::NonNull::from(value).cast();
                let raw_window_handle = raw_window_handle::WebCanvasWindowHandle::new(obj).into();
                let raw_display_handle = raw_window_handle::WebDisplayHandle::new().into();

                // Note that we need to call this while we still have `value` around.
                // This is safe without storing canvas to `handle_origin` since the surface will create a copy internally.
                unsafe {
                    self.create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle,
                        raw_window_handle,
                    })
                }?
            }

            #[cfg(any(webgpu, webgl))]
            SurfaceTarget::OffscreenCanvas(canvas) => {

                let value: &wasm_bindgen::JsValue = &canvas;
                let obj = std::ptr::NonNull::from(value).cast();
                let raw_window_handle =
                    raw_window_handle::WebOffscreenCanvasWindowHandle::new(obj).into();
                let raw_display_handle = raw_window_handle::WebDisplayHandle::new().into();

                // Note that we need to call this while we still have `value` around.
                // This is safe without storing canvas to `handle_origin` since the surface will create a copy internally.
                unsafe {
                    self.create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle,
                        raw_window_handle,
                    })
                }?
            }
        };

		log::trace!("pi_wgpu::Instance::create_surface, result = {:?}", surface);
        Ok(super::super::Surface {
            inner: surface.unwrap(),
            window: target,
        })
    }
}


/// The window/canvas/surface/swap-chain/etc. a surface is attached to, for use with unsafe surface creation.
///
/// This is either a window or an actual web canvas depending on the platform and
/// enabled features.
/// Refer to the individual variants for more information.
///
/// See also [`SurfaceTarget`] for safe variants.
pub struct RawHandle<'a> {
    /// Raw display handle, underlying display must outlive the surface created from this.
    raw_display_handle: DisplayHandle<'a>,

    /// Raw display handle, underlying window must outlive the surface created from this.
    raw_window_handle: WindowHandle<'a>,
}

impl<'a> RawHandle<'a> {
    /// Creates a [`SurfaceTargetUnsafe::RawHandle`] from a window.
    ///
    /// # Safety
    ///
    /// - `window` must outlive the resulting surface target
    ///   (and subsequently the surface created for this target).
    pub unsafe fn from_window(window: &'a Box<dyn super::surface::WindowHandle>) -> Result<Self, raw_window_handle::HandleError>
    {
        Ok(Self {
            raw_display_handle: window.display_handle()?,
            raw_window_handle: window.window_handle()?,
        })
    }
}

impl<'a> HasWindowHandle for RawHandle<'a> {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        Ok(self.raw_window_handle.clone())
    }
}

impl<'a> HasDisplayHandle for RawHandle<'a> {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(self.raw_display_handle.clone())
    }
}


