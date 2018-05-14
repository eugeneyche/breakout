use gl;
use gl::types::*;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::fs::File;
use std::ffi::CString;
use std::io;
use std::ptr;
use std::str;

#[derive(Copy, Clone, Debug)]
pub struct Shader {
    pub id: GLuint,
}

#[derive(Copy, Clone, Debug)]
pub struct Program {
    pub id: GLuint,
}

#[derive(Debug)]
pub enum CreateShaderError {
    Io(PathBuf, io::Error),
    Compile(PathBuf, String),
}

#[derive(Debug)]
pub enum CreateProgramError {
    Link(String),
}

impl Shader {
    pub fn from_file<P: AsRef<Path>>(ty: GLenum, path: P) -> Result<Self, CreateShaderError> {
        let path = path.as_ref();
        let mut contents = String::new();
        let mut file = File::open(path).map_err(|err| CreateShaderError::Io(path.into(), err))?; 
        file.read_to_string(&mut contents).map_err(|err| CreateShaderError::Io(path.into(), err))?;
        let contents_len = contents.len();
        let c_contents = CString::new(contents).unwrap();
        unsafe {
            let id = gl::CreateShader(ty);
            gl::ShaderSource(id, 1, &c_contents.as_ptr(), &(contents_len as _));
            gl::CompileShader(id);
            let mut is_compiled = 0;
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut is_compiled);
            if is_compiled == 0 {
                let mut buffer = vec![0; 1024];
                gl::GetShaderInfoLog(id, 1024, ptr::null_mut(), buffer.as_mut_ptr() as _);
                let info_log = str::from_utf8(&buffer).unwrap();
                return Err(CreateShaderError::Compile(path.into(), info_log.into()));
            } else {
                Ok(Shader { id })
            }
        }
    }
}

impl Program {
    pub fn new(shaders: &[Shader]) -> Result<Self, CreateProgramError> { unsafe {
        let id = gl::CreateProgram();
        for &shader in shaders {
            gl::AttachShader(id, shader.id);
        }
        gl::LinkProgram(id);
        for &shader in shaders {
            gl::DetachShader(id, shader.id);
        }
        let mut is_linked = 0;
        gl::GetProgramiv(id, gl::LINK_STATUS, &mut is_linked);
        if is_linked == 0 {
            let mut buffer = vec![0; 1024];
            gl::GetProgramInfoLog(id, 1024, ptr::null_mut(), buffer.as_mut_ptr() as _);
            let info_log = str::from_utf8(&buffer).unwrap();
            Err(CreateProgramError::Link(info_log.into()))
        } else {
            Ok(Program { id })
        }
    }}

    pub fn get_uniform_location(&self, name: &str) -> GLint { unsafe {
        gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr())
    }}
}
