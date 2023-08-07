use crate::{Backend, Instance, Surface};

//TODO: remove this
pub struct HalSurface<A: crate::wgpu_hal::Api> {
    pub raw: A::Surface,
}

pub trait HalApi: crate::wgpu_hal::Api {
    const VARIANT: Backend;

    fn create_instance_from_hal(hal_instance: Self::Instance) -> Instance;

    fn instance_as_hal(instance: &Instance) -> Option<&Self::Instance>;

    fn get_surface(surface: &Surface) -> Option<&HalSurface<Self>>;

    fn get_surface_mut(surface: &mut Surface) -> Option<&mut HalSurface<Self>>;
}

#[cfg(feature = "empty")]
impl HalApi for crate::wgpu_hal::Empty {
    const VARIANT: Backend = Backend::Empty;

    fn create_instance_from_hal(hal_instance: Self::Instance) -> Instance {
        unimplemented!("Empty::create_instance_from_hal is not implemented")
    }

    fn instance_as_hal(instance: &Instance) -> Option<&Self::Instance> {
        unimplemented!("Empty::instance_as_hal is not implemented")
    }

    fn get_surface(surface: &Surface) -> Option<&HalSurface<Self>> {
        unimplemented!("Empty::get_surface is not implemented")
    }

    fn get_surface_mut(surface: &mut Surface) -> Option<&mut HalSurface<Self>> {
        unimplemented!("Empty::get_surface_mut is not implemented")
    }
}

#[cfg(feature = "gl")]
impl HalApi for crate::wgpu_hal::GL {
    const VARIANT: Backend = Backend::Gl;

    fn create_instance_from_hal(hal_instance: Self::Instance) -> Instance {
        unimplemented!("GL::create_instance_from_hal is not implemented")
    }

    fn instance_as_hal(instance: &Instance) -> Option<&Self::Instance> {
        unimplemented!("GL::instance_as_hal is not implemented")
    }

    fn get_surface(surface: &Surface) -> Option<&HalSurface<Self>> {
        unimplemented!("GL::get_surface is not implemented")
    }

    fn get_surface_mut(surface: &mut Surface) -> Option<&mut HalSurface<Self>> {
        unimplemented!("GL::get_surface_mut is not implemented")
    }
}
