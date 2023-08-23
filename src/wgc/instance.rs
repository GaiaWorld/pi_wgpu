use std::{
    future::{ready, Future},
    iter::Iterator,
    sync::Arc,
};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::{hal, wgt, Backends, CreateSurfaceError, InstanceDescriptor, PowerPreference, Surface};

/// Context for all other wgpu objects. Instance of wgpu.
///
/// This is the first thing you create when using wgpu.
/// Its primary use is to create [`Adapter`]s and [`Surface`]s.
///
/// Does not have to be kept alive.
///
/// Corresponds to [WebGPU `GPU`](https://gpuweb.github.io/gpuweb/#gpu-interface).
#[derive(Debug, Clone)]
pub struct Instance(pub(crate) Arc<hal::Instance>);

impl Default for Instance {
    /// Creates a new instance of wgpu with default options.
    ///
    /// Backends are set to `Backends::all()`, and FXC is chosen as the `dx12_shader_compiler`.
    fn default() -> Self {
        Instance::new(InstanceDescriptor {
            backends: Backends::GL,
            dx12_shader_compiler: wgt::Dx12Compiler::default(),
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
    pub fn new(instance_desc: InstanceDescriptor) -> Self {
        profiling::scope!("Instance::new");

        assert!(instance_desc.backends.contains(Backends::GL));

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

        let imp = unsafe { hal::Instance::init(&hal_desc).unwrap() };

        Self(Arc::new(imp))
    }

    /// Create an new instance of wgpu from a wgpu-hal instance.
    ///
    /// # Arguments
    ///
    /// - `hal_instance` - wgpu-hal instance.
    ///
    /// # Safety
    ///
    /// Refer to the creation of wgpu-hal Instance for every backend.
    #[cfg(any(not(target_arch = "wasm32"), feature = "emscripten"))]
    pub unsafe fn from_hal<A: crate::HalApi>(hal_instance: A::Instance) -> Self {
        unimplemented!("Instance::from_hal is not implemented")
    }

    /// Return a reference to a specific backend instance, if available.
    ///
    /// If this `Instance` has a wgpu-hal [`Instance`] for backend
    /// `A`, return a reference to it. Otherwise, return `None`.
    ///
    /// # Safety
    ///
    /// - The raw instance handle returned must not be manually destroyed.
    ///
    /// [`Instance`]: crate::wgpu_Api::Instance
    #[cfg(any(not(target_arch = "wasm32"), feature = "webgl"))]
    pub unsafe fn as_hal<A: crate::HalApi>(&self) -> Option<&A::Instance> {
        unimplemented!("Instance::as_hal is not implemented")
    }

    /// Retrieves all available [`Adapter`]s that match the given [`Backends`].
    ///
    /// # Arguments
    ///
    /// - `backends` - Backends from which to enumerate adapters.
    #[cfg(any(not(target_arch = "wasm32"), feature = "emscripten"))]
    pub fn enumerate_adapters(&self, backends: Backends) -> impl Iterator<Item = crate::Adapter> {
        unimplemented!("Instance::enumerate_adapters is not implemented");

        #[allow(unreachable_code)]
        vec![].into_iter()
    }

    /// Retrieves an [`Adapter`] which matches the given [`RequestAdapterOptions`].
    ///
    /// Some options are "soft", so treated as non-mandatory. Others are "hard".
    ///
    /// If no adapters are found that suffice all the "hard" options, `None` is returned.
    pub fn request_adapter(
        &self,
        options: &crate::RequestAdapterOptions,
    ) -> impl Future<Output = Option<crate::Adapter>> + Send {
        profiling::scope!("Instance::request_adapter");

        // 不支持 软件 Adapter
        assert!(!options.force_fallback_adapter);

        let adapters = unsafe { self.0.enumerate_adapters() };

        if let Some(surface) = options.compatible_surface {
            let surface = &surface.inner;

            adapters.retain(|exposed| unsafe {
                exposed.adapter.surface_capabilities(&surface).is_some()
            });
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
                return ready(Some(adapter));
            }
        }

        return ready(None);
    }

    /// Converts a wgpu-hal `ExposedAdapter` to a wgpu [`Adapter`].
    ///
    /// # Safety
    ///
    /// `hal_adapter` must be created from this instance internal handle.
    #[cfg(any(not(target_arch = "wasm32"), feature = "emscripten"))]
    pub unsafe fn create_adapter_from_hal<A: crate::HalApi>(
        &self,
        hal_adapter: crate::ExposedAdapter<A>,
    ) -> crate::Adapter {
        unimplemented!("Instance::create_adapter_from_hal is not implemented")
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
    pub unsafe fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<Surface, CreateSurfaceError> {
        profiling::scope!("Instance::create_surface");

        let display_handle = HasRawDisplayHandle::raw_display_handle(window);

        let window_handle = HasRawWindowHandle::raw_window_handle(window);

        let raw = self.0.create_surface(display_handle, window_handle);

        Ok(crate::Surface {
            inner: raw.unwrap(),
            instance: self.clone(),
        })
    }

    /// Creates a surface from `CoreAnimationLayer`.
    ///
    /// # Safety
    ///
    /// - layer must be a valid object to create a surface upon.
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub unsafe fn create_surface_from_core_animation_layer(
        &self,
        layer: *mut std::ffi::c_void,
    ) -> Surface {
        unimplemented!("Instance::create_surface_from_core_animation_layer is not implemented")
    }

    /// Creates a surface from `IDCompositionVisual`.
    ///
    /// # Safety
    ///
    /// - visual must be a valid IDCompositionVisual to create a surface upon.
    #[cfg(target_os = "windows")]
    pub unsafe fn create_surface_from_visual(&self, visual: *mut std::ffi::c_void) -> Surface {
        profiling::scope!("Instance::create_surface_from_visual");

        unimplemented!("Instance::create_surface_from_visual is not implemented")
    }

    /// Creates a surface from `SurfaceHandle`.
    ///
    /// # Safety
    ///
    /// - surface_handle must be a valid SurfaceHandle to create a surface upon.
    #[cfg(target_os = "windows")]
    pub unsafe fn create_surface_from_surface_handle(
        &self,
        surface_handle: *mut std::ffi::c_void,
    ) -> Surface {
        profiling::scope!("Instance::create_surface_from_surface_handle");

        unimplemented!("Instance::create_surface_from_surface_handle is not implemented")
    }

    /// Creates a surface from a `web_sys::HtmlCanvasElement`.
    ///
    /// The `canvas` argument must be a valid `<canvas>` element to
    /// create a surface upon.
    ///
    /// # Errors
    ///
    /// - On WebGL2: Will return an error if the browser does not support WebGL2,
    ///   or declines to provide GPU access (such as due to a resource shortage).
    #[cfg(all(target_arch = "wasm32", not(feature = "emscripten")))]
    pub fn create_surface_from_canvas(
        &self,
        canvas: &web_sys::HtmlCanvasElement,
    ) -> Result<Surface, CreateSurfaceError> {
        profiling::scope!("Instance::create_surface_from_canvas");

        unimplemented!("Instance::create_surface_from_canvas is not implemented")
    }

    /// Creates a surface from a `web_sys::OffscreenCanvas`.
    ///
    /// The `canvas` argument must be a valid `OffscreenCanvas` object
    /// to create a surface upon.
    ///
    /// # Errors
    ///
    /// - On WebGL2: Will return an error if the browser does not support WebGL2,
    ///   or declines to provide GPU access (such as due to a resource shortage).
    #[cfg(all(target_arch = "wasm32", not(feature = "emscripten")))]
    pub fn create_surface_from_offscreen_canvas(
        &self,
        canvas: &web_sys::OffscreenCanvas,
    ) -> Result<Surface, CreateSurfaceError> {
        profiling::scope!("Instance::create_surface_from_offscreen_canvas");

        unimplemented!("Instance::create_surface_from_offscreen_canvas is not implemented")
    }

    /// Polls all devices.
    ///
    /// If `force_wait` is true and this is not running on the web, then this
    /// function will block until all in-flight buffers have been mapped and
    /// all submitted commands have finished execution.
    ///
    /// Return `true` if all devices' queues are empty, or `false` if there are
    /// queue submissions still in flight. (Note that, unless access to all
    /// [`Queue`s] associated with this [`Instance`] is coordinated somehow,
    /// this information could be out of date by the time the caller receives
    /// it. `Queue`s can be shared between threads, and other threads could
    /// submit new work at any time.)
    ///
    /// On the web, this is a no-op. `Device`s are automatically polled.
    ///
    /// [`Queue`s]: Queue
    pub fn poll_all(&self, force_wait: bool) -> bool {
        profiling::scope!("Instance::poll_all");

        unimplemented!("Instance::poll_all is not implemented")
    }
}
