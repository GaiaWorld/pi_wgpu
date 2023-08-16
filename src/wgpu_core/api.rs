use crate::{Backend, Instance, Surface};

pub trait HalApi: crate::wgpu_hal::Api {
    const VARIANT: Backend;

    fn create_instance_from_hal(hal_instance: Self::Instance) -> Instance;

    fn instance_as_hal(instance: &Instance) -> Option<&Self::Instance>;

    fn get_surface(surface: &Surface) -> Option<&Self::Surface>;

    fn get_surface_mut(surface: &mut Surface) -> Option<&mut Self::Surface>;
}

impl HalApi for crate::wgpu_hal::Empty {
    const VARIANT: Backend = Backend::Empty;

    fn create_instance_from_hal(hal_instance: Self::Instance) -> Instance {
        unimplemented!("Empty::create_instance_from_hal is not implemented")
    }

    fn instance_as_hal(instance: &Instance) -> Option<&Self::Instance> {
        unimplemented!("Empty::instance_as_hal is not implemented")
    }

    fn get_surface(surface: &Surface) -> Option<&Self::Surface> {
        unimplemented!("Empty::get_surface is not implemented")
    }

    fn get_surface_mut(surface: &mut Surface) -> Option<&mut Self::Surface> {
        unimplemented!("Empty::get_surface_mut is not implemented")
    }
}

#[cfg(feature = "gl")]
impl HalApi for crate::wgpu_hal::GL {
    const VARIANT: Backend = Backend::Gl;
    fn create_instance_from_hal(hal_instance: Self::Instance) -> Instance {
        #[allow(clippy::needless_update)]
        Instance {
            inner: hal_instance,
        }
    }

    fn instance_as_hal(instance: &Instance) -> Option<&Self::Instance> {
        Some(&instance.inner)
    }

    fn get_surface(surface: &Surface) -> Option<&Self::Surface> {
        Some(&surface.inner)
    }
    fn get_surface_mut(surface: &mut Surface) -> Option<&mut Self::Surface> {
        Some(&mut surface.inner)
    }
}
