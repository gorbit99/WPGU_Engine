use shaderc::{Compiler, ShaderKind};

pub struct Shader {
    pub(super) bytes: Vec<u32>
}

impl Shader {
    pub fn new_from_source(source: &str, kind: ShaderKind) -> Shader {
        let mut compiler = Compiler::new().unwrap();
        let result =
            compiler.compile_into_spirv(source, kind, "shader.glsl", "main", None).unwrap();
        Shader {
            bytes: result.as_binary().to_vec()
        }
    }
}