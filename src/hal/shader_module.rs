use super::GLState;

#[derive(Debug)]
pub(crate) struct ShaderModule {

}

impl ShaderModule {
    pub fn new(
        state: GLState,
        desc: &crate::ShaderModuleDescriptor,
    ) -> Result<Self, super::ShaderError> {
    }
}