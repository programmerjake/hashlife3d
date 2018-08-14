// This file is part of Hashlife3d.
//
// Hashlife3d is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Hashlife3d is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with Hashlife3d.  If not, see <https://www.gnu.org/licenses/>
use super::super::sdl;
use super::*;
use std::error;
use std::ffi::{CStr, CString};
use std::fmt;
use std::mem;
use std::os::raw::*;
use std::ptr::*;
use std::result;
use std::sync::*;

#[allow(dead_code)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod api {
    include!(concat!(env!("OUT_DIR"), "/gles2-bindings.rs"));
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct Api {
    pub glActiveTexture: api::PFNGLACTIVETEXTUREPROC,
    pub glAttachShader: api::PFNGLATTACHSHADERPROC,
    pub glBindAttribLocation: api::PFNGLBINDATTRIBLOCATIONPROC,
    pub glBindBuffer: api::PFNGLBINDBUFFERPROC,
    pub glBindFramebuffer: api::PFNGLBINDFRAMEBUFFERPROC,
    pub glBindRenderbuffer: api::PFNGLBINDRENDERBUFFERPROC,
    pub glBindTexture: api::PFNGLBINDTEXTUREPROC,
    pub glBlendColor: api::PFNGLBLENDCOLORPROC,
    pub glBlendEquation: api::PFNGLBLENDEQUATIONPROC,
    pub glBlendEquationSeparate: api::PFNGLBLENDEQUATIONSEPARATEPROC,
    pub glBlendFunc: api::PFNGLBLENDFUNCPROC,
    pub glBlendFuncSeparate: api::PFNGLBLENDFUNCSEPARATEPROC,
    pub glBufferData: api::PFNGLBUFFERDATAPROC,
    pub glBufferSubData: api::PFNGLBUFFERSUBDATAPROC,
    pub glCheckFramebufferStatus: api::PFNGLCHECKFRAMEBUFFERSTATUSPROC,
    pub glClear: api::PFNGLCLEARPROC,
    pub glClearColor: api::PFNGLCLEARCOLORPROC,
    pub glClearDepthf: api::PFNGLCLEARDEPTHFPROC,
    pub glClearStencil: api::PFNGLCLEARSTENCILPROC,
    pub glColorMask: api::PFNGLCOLORMASKPROC,
    pub glCompileShader: api::PFNGLCOMPILESHADERPROC,
    pub glCompressedTexImage2D: api::PFNGLCOMPRESSEDTEXIMAGE2DPROC,
    pub glCompressedTexSubImage2D: api::PFNGLCOMPRESSEDTEXSUBIMAGE2DPROC,
    pub glCopyTexImage2D: api::PFNGLCOPYTEXIMAGE2DPROC,
    pub glCopyTexSubImage2D: api::PFNGLCOPYTEXSUBIMAGE2DPROC,
    pub glCreateProgram: api::PFNGLCREATEPROGRAMPROC,
    pub glCreateShader: api::PFNGLCREATESHADERPROC,
    pub glCullFace: api::PFNGLCULLFACEPROC,
    pub glDeleteBuffers: api::PFNGLDELETEBUFFERSPROC,
    pub glDeleteFramebuffers: api::PFNGLDELETEFRAMEBUFFERSPROC,
    pub glDeleteProgram: api::PFNGLDELETEPROGRAMPROC,
    pub glDeleteRenderbuffers: api::PFNGLDELETERENDERBUFFERSPROC,
    pub glDeleteShader: api::PFNGLDELETESHADERPROC,
    pub glDeleteTextures: api::PFNGLDELETETEXTURESPROC,
    pub glDepthFunc: api::PFNGLDEPTHFUNCPROC,
    pub glDepthMask: api::PFNGLDEPTHMASKPROC,
    pub glDepthRangef: api::PFNGLDEPTHRANGEFPROC,
    pub glDetachShader: api::PFNGLDETACHSHADERPROC,
    pub glDisable: api::PFNGLDISABLEPROC,
    pub glDisableVertexAttribArray: api::PFNGLDISABLEVERTEXATTRIBARRAYPROC,
    pub glDrawArrays: api::PFNGLDRAWARRAYSPROC,
    pub glDrawElements: api::PFNGLDRAWELEMENTSPROC,
    pub glEnable: api::PFNGLENABLEPROC,
    pub glEnableVertexAttribArray: api::PFNGLENABLEVERTEXATTRIBARRAYPROC,
    pub glFinish: api::PFNGLFINISHPROC,
    pub glFlush: api::PFNGLFLUSHPROC,
    pub glFramebufferRenderbuffer: api::PFNGLFRAMEBUFFERRENDERBUFFERPROC,
    pub glFramebufferTexture2D: api::PFNGLFRAMEBUFFERTEXTURE2DPROC,
    pub glFrontFace: api::PFNGLFRONTFACEPROC,
    pub glGenBuffers: api::PFNGLGENBUFFERSPROC,
    pub glGenerateMipmap: api::PFNGLGENERATEMIPMAPPROC,
    pub glGenFramebuffers: api::PFNGLGENFRAMEBUFFERSPROC,
    pub glGenRenderbuffers: api::PFNGLGENRENDERBUFFERSPROC,
    pub glGenTextures: api::PFNGLGENTEXTURESPROC,
    pub glGetActiveAttrib: api::PFNGLGETACTIVEATTRIBPROC,
    pub glGetActiveUniform: api::PFNGLGETACTIVEUNIFORMPROC,
    pub glGetAttachedShaders: api::PFNGLGETATTACHEDSHADERSPROC,
    pub glGetAttribLocation: api::PFNGLGETATTRIBLOCATIONPROC,
    pub glGetBooleanv: api::PFNGLGETBOOLEANVPROC,
    pub glGetBufferParameteriv: api::PFNGLGETBUFFERPARAMETERIVPROC,
    pub glGetError: api::PFNGLGETERRORPROC,
    pub glGetFloatv: api::PFNGLGETFLOATVPROC,
    pub glGetFramebufferAttachmentParameteriv: api::PFNGLGETFRAMEBUFFERATTACHMENTPARAMETERIVPROC,
    pub glGetIntegerv: api::PFNGLGETINTEGERVPROC,
    pub glGetProgramiv: api::PFNGLGETPROGRAMIVPROC,
    pub glGetProgramInfoLog: api::PFNGLGETPROGRAMINFOLOGPROC,
    pub glGetRenderbufferParameteriv: api::PFNGLGETRENDERBUFFERPARAMETERIVPROC,
    pub glGetShaderiv: api::PFNGLGETSHADERIVPROC,
    pub glGetShaderInfoLog: api::PFNGLGETSHADERINFOLOGPROC,
    pub glGetShaderPrecisionFormat: api::PFNGLGETSHADERPRECISIONFORMATPROC,
    pub glGetShaderSource: api::PFNGLGETSHADERSOURCEPROC,
    pub glGetString: api::PFNGLGETSTRINGPROC,
    pub glGetTexParameterfv: api::PFNGLGETTEXPARAMETERFVPROC,
    pub glGetTexParameteriv: api::PFNGLGETTEXPARAMETERIVPROC,
    pub glGetUniformfv: api::PFNGLGETUNIFORMFVPROC,
    pub glGetUniformiv: api::PFNGLGETUNIFORMIVPROC,
    pub glGetUniformLocation: api::PFNGLGETUNIFORMLOCATIONPROC,
    pub glGetVertexAttribfv: api::PFNGLGETVERTEXATTRIBFVPROC,
    pub glGetVertexAttribiv: api::PFNGLGETVERTEXATTRIBIVPROC,
    pub glGetVertexAttribPointerv: api::PFNGLGETVERTEXATTRIBPOINTERVPROC,
    pub glHint: api::PFNGLHINTPROC,
    pub glIsBuffer: api::PFNGLISBUFFERPROC,
    pub glIsEnabled: api::PFNGLISENABLEDPROC,
    pub glIsFramebuffer: api::PFNGLISFRAMEBUFFERPROC,
    pub glIsProgram: api::PFNGLISPROGRAMPROC,
    pub glIsRenderbuffer: api::PFNGLISRENDERBUFFERPROC,
    pub glIsShader: api::PFNGLISSHADERPROC,
    pub glIsTexture: api::PFNGLISTEXTUREPROC,
    pub glLineWidth: api::PFNGLLINEWIDTHPROC,
    pub glLinkProgram: api::PFNGLLINKPROGRAMPROC,
    pub glPixelStorei: api::PFNGLPIXELSTOREIPROC,
    pub glPolygonOffset: api::PFNGLPOLYGONOFFSETPROC,
    pub glReadPixels: api::PFNGLREADPIXELSPROC,
    pub glReleaseShaderCompiler: api::PFNGLRELEASESHADERCOMPILERPROC,
    pub glRenderbufferStorage: api::PFNGLRENDERBUFFERSTORAGEPROC,
    pub glSampleCoverage: api::PFNGLSAMPLECOVERAGEPROC,
    pub glScissor: api::PFNGLSCISSORPROC,
    pub glShaderBinary: api::PFNGLSHADERBINARYPROC,
    pub glShaderSource: api::PFNGLSHADERSOURCEPROC,
    pub glStencilFunc: api::PFNGLSTENCILFUNCPROC,
    pub glStencilFuncSeparate: api::PFNGLSTENCILFUNCSEPARATEPROC,
    pub glStencilMask: api::PFNGLSTENCILMASKPROC,
    pub glStencilMaskSeparate: api::PFNGLSTENCILMASKSEPARATEPROC,
    pub glStencilOp: api::PFNGLSTENCILOPPROC,
    pub glStencilOpSeparate: api::PFNGLSTENCILOPSEPARATEPROC,
    pub glTexImage2D: api::PFNGLTEXIMAGE2DPROC,
    pub glTexParameterf: api::PFNGLTEXPARAMETERFPROC,
    pub glTexParameterfv: api::PFNGLTEXPARAMETERFVPROC,
    pub glTexParameteri: api::PFNGLTEXPARAMETERIPROC,
    pub glTexParameteriv: api::PFNGLTEXPARAMETERIVPROC,
    pub glTexSubImage2D: api::PFNGLTEXSUBIMAGE2DPROC,
    pub glUniform1f: api::PFNGLUNIFORM1FPROC,
    pub glUniform1fv: api::PFNGLUNIFORM1FVPROC,
    pub glUniform1i: api::PFNGLUNIFORM1IPROC,
    pub glUniform1iv: api::PFNGLUNIFORM1IVPROC,
    pub glUniform2f: api::PFNGLUNIFORM2FPROC,
    pub glUniform2fv: api::PFNGLUNIFORM2FVPROC,
    pub glUniform2i: api::PFNGLUNIFORM2IPROC,
    pub glUniform2iv: api::PFNGLUNIFORM2IVPROC,
    pub glUniform3f: api::PFNGLUNIFORM3FPROC,
    pub glUniform3fv: api::PFNGLUNIFORM3FVPROC,
    pub glUniform3i: api::PFNGLUNIFORM3IPROC,
    pub glUniform3iv: api::PFNGLUNIFORM3IVPROC,
    pub glUniform4f: api::PFNGLUNIFORM4FPROC,
    pub glUniform4fv: api::PFNGLUNIFORM4FVPROC,
    pub glUniform4i: api::PFNGLUNIFORM4IPROC,
    pub glUniform4iv: api::PFNGLUNIFORM4IVPROC,
    pub glUniformMatrix2fv: api::PFNGLUNIFORMMATRIX2FVPROC,
    pub glUniformMatrix3fv: api::PFNGLUNIFORMMATRIX3FVPROC,
    pub glUniformMatrix4fv: api::PFNGLUNIFORMMATRIX4FVPROC,
    pub glUseProgram: api::PFNGLUSEPROGRAMPROC,
    pub glValidateProgram: api::PFNGLVALIDATEPROGRAMPROC,
    pub glVertexAttrib1f: api::PFNGLVERTEXATTRIB1FPROC,
    pub glVertexAttrib1fv: api::PFNGLVERTEXATTRIB1FVPROC,
    pub glVertexAttrib2f: api::PFNGLVERTEXATTRIB2FPROC,
    pub glVertexAttrib2fv: api::PFNGLVERTEXATTRIB2FVPROC,
    pub glVertexAttrib3f: api::PFNGLVERTEXATTRIB3FPROC,
    pub glVertexAttrib3fv: api::PFNGLVERTEXATTRIB3FVPROC,
    pub glVertexAttrib4f: api::PFNGLVERTEXATTRIB4FPROC,
    pub glVertexAttrib4fv: api::PFNGLVERTEXATTRIB4FVPROC,
    pub glVertexAttribPointer: api::PFNGLVERTEXATTRIBPOINTERPROC,
    pub glViewport: api::PFNGLVIEWPORTPROC,
}

unsafe fn get_fn(name: &[u8]) -> *const c_void {
    let name = CStr::from_bytes_with_nul(name).unwrap();
    let retval = sdl::api::SDL_GL_GetProcAddress(name.as_ptr());
    if retval.is_null() {
        panic!(
            "SDL_GL_GetProcAddress failed looking for {}: {}",
            name.to_string_lossy(),
            sdl::get_error()
        );
    }
    retval
}

macro_rules! get_fn {
    ($name:ident, $type:ident) => {{
        let f: api::$type = mem::transmute(get_fn(concat!(stringify!($name), "\0").as_bytes()));
        match f {
            Some(retval) => Some(retval),
            None => panic!("SDL_GL_GetProcAddress failed: {}", sdl::get_error()),
        }
    }};
}

impl Api {
    pub unsafe fn new() -> Self {
        Self {
            glActiveTexture: get_fn!(glActiveTexture, PFNGLACTIVETEXTUREPROC),
            glAttachShader: get_fn!(glAttachShader, PFNGLATTACHSHADERPROC),
            glBindAttribLocation: get_fn!(glBindAttribLocation, PFNGLBINDATTRIBLOCATIONPROC),
            glBindBuffer: get_fn!(glBindBuffer, PFNGLBINDBUFFERPROC),
            glBindFramebuffer: get_fn!(glBindFramebuffer, PFNGLBINDFRAMEBUFFERPROC),
            glBindRenderbuffer: get_fn!(glBindRenderbuffer, PFNGLBINDRENDERBUFFERPROC),
            glBindTexture: get_fn!(glBindTexture, PFNGLBINDTEXTUREPROC),
            glBlendColor: get_fn!(glBlendColor, PFNGLBLENDCOLORPROC),
            glBlendEquation: get_fn!(glBlendEquation, PFNGLBLENDEQUATIONPROC),
            glBlendEquationSeparate: get_fn!(
                glBlendEquationSeparate,
                PFNGLBLENDEQUATIONSEPARATEPROC
            ),
            glBlendFunc: get_fn!(glBlendFunc, PFNGLBLENDFUNCPROC),
            glBlendFuncSeparate: get_fn!(glBlendFuncSeparate, PFNGLBLENDFUNCSEPARATEPROC),
            glBufferData: get_fn!(glBufferData, PFNGLBUFFERDATAPROC),
            glBufferSubData: get_fn!(glBufferSubData, PFNGLBUFFERSUBDATAPROC),
            glCheckFramebufferStatus: get_fn!(
                glCheckFramebufferStatus,
                PFNGLCHECKFRAMEBUFFERSTATUSPROC
            ),
            glClear: get_fn!(glClear, PFNGLCLEARPROC),
            glClearColor: get_fn!(glClearColor, PFNGLCLEARCOLORPROC),
            glClearDepthf: get_fn!(glClearDepthf, PFNGLCLEARDEPTHFPROC),
            glClearStencil: get_fn!(glClearStencil, PFNGLCLEARSTENCILPROC),
            glColorMask: get_fn!(glColorMask, PFNGLCOLORMASKPROC),
            glCompileShader: get_fn!(glCompileShader, PFNGLCOMPILESHADERPROC),
            glCompressedTexImage2D: get_fn!(glCompressedTexImage2D, PFNGLCOMPRESSEDTEXIMAGE2DPROC),
            glCompressedTexSubImage2D: get_fn!(
                glCompressedTexSubImage2D,
                PFNGLCOMPRESSEDTEXSUBIMAGE2DPROC
            ),
            glCopyTexImage2D: get_fn!(glCopyTexImage2D, PFNGLCOPYTEXIMAGE2DPROC),
            glCopyTexSubImage2D: get_fn!(glCopyTexSubImage2D, PFNGLCOPYTEXSUBIMAGE2DPROC),
            glCreateProgram: get_fn!(glCreateProgram, PFNGLCREATEPROGRAMPROC),
            glCreateShader: get_fn!(glCreateShader, PFNGLCREATESHADERPROC),
            glCullFace: get_fn!(glCullFace, PFNGLCULLFACEPROC),
            glDeleteBuffers: get_fn!(glDeleteBuffers, PFNGLDELETEBUFFERSPROC),
            glDeleteFramebuffers: get_fn!(glDeleteFramebuffers, PFNGLDELETEFRAMEBUFFERSPROC),
            glDeleteProgram: get_fn!(glDeleteProgram, PFNGLDELETEPROGRAMPROC),
            glDeleteRenderbuffers: get_fn!(glDeleteRenderbuffers, PFNGLDELETERENDERBUFFERSPROC),
            glDeleteShader: get_fn!(glDeleteShader, PFNGLDELETESHADERPROC),
            glDeleteTextures: get_fn!(glDeleteTextures, PFNGLDELETETEXTURESPROC),
            glDepthFunc: get_fn!(glDepthFunc, PFNGLDEPTHFUNCPROC),
            glDepthMask: get_fn!(glDepthMask, PFNGLDEPTHMASKPROC),
            glDepthRangef: get_fn!(glDepthRangef, PFNGLDEPTHRANGEFPROC),
            glDetachShader: get_fn!(glDetachShader, PFNGLDETACHSHADERPROC),
            glDisable: get_fn!(glDisable, PFNGLDISABLEPROC),
            glDisableVertexAttribArray: get_fn!(
                glDisableVertexAttribArray,
                PFNGLDISABLEVERTEXATTRIBARRAYPROC
            ),
            glDrawArrays: get_fn!(glDrawArrays, PFNGLDRAWARRAYSPROC),
            glDrawElements: get_fn!(glDrawElements, PFNGLDRAWELEMENTSPROC),
            glEnable: get_fn!(glEnable, PFNGLENABLEPROC),
            glEnableVertexAttribArray: get_fn!(
                glEnableVertexAttribArray,
                PFNGLENABLEVERTEXATTRIBARRAYPROC
            ),
            glFinish: get_fn!(glFinish, PFNGLFINISHPROC),
            glFlush: get_fn!(glFlush, PFNGLFLUSHPROC),
            glFramebufferRenderbuffer: get_fn!(
                glFramebufferRenderbuffer,
                PFNGLFRAMEBUFFERRENDERBUFFERPROC
            ),
            glFramebufferTexture2D: get_fn!(glFramebufferTexture2D, PFNGLFRAMEBUFFERTEXTURE2DPROC),
            glFrontFace: get_fn!(glFrontFace, PFNGLFRONTFACEPROC),
            glGenBuffers: get_fn!(glGenBuffers, PFNGLGENBUFFERSPROC),
            glGenerateMipmap: get_fn!(glGenerateMipmap, PFNGLGENERATEMIPMAPPROC),
            glGenFramebuffers: get_fn!(glGenFramebuffers, PFNGLGENFRAMEBUFFERSPROC),
            glGenRenderbuffers: get_fn!(glGenRenderbuffers, PFNGLGENRENDERBUFFERSPROC),
            glGenTextures: get_fn!(glGenTextures, PFNGLGENTEXTURESPROC),
            glGetActiveAttrib: get_fn!(glGetActiveAttrib, PFNGLGETACTIVEATTRIBPROC),
            glGetActiveUniform: get_fn!(glGetActiveUniform, PFNGLGETACTIVEUNIFORMPROC),
            glGetAttachedShaders: get_fn!(glGetAttachedShaders, PFNGLGETATTACHEDSHADERSPROC),
            glGetAttribLocation: get_fn!(glGetAttribLocation, PFNGLGETATTRIBLOCATIONPROC),
            glGetBooleanv: get_fn!(glGetBooleanv, PFNGLGETBOOLEANVPROC),
            glGetBufferParameteriv: get_fn!(glGetBufferParameteriv, PFNGLGETBUFFERPARAMETERIVPROC),
            glGetError: get_fn!(glGetError, PFNGLGETERRORPROC),
            glGetFloatv: get_fn!(glGetFloatv, PFNGLGETFLOATVPROC),
            glGetFramebufferAttachmentParameteriv: get_fn!(
                glGetFramebufferAttachmentParameteriv,
                PFNGLGETFRAMEBUFFERATTACHMENTPARAMETERIVPROC
            ),
            glGetIntegerv: get_fn!(glGetIntegerv, PFNGLGETINTEGERVPROC),
            glGetProgramiv: get_fn!(glGetProgramiv, PFNGLGETPROGRAMIVPROC),
            glGetProgramInfoLog: get_fn!(glGetProgramInfoLog, PFNGLGETPROGRAMINFOLOGPROC),
            glGetRenderbufferParameteriv: get_fn!(
                glGetRenderbufferParameteriv,
                PFNGLGETRENDERBUFFERPARAMETERIVPROC
            ),
            glGetShaderiv: get_fn!(glGetShaderiv, PFNGLGETSHADERIVPROC),
            glGetShaderInfoLog: get_fn!(glGetShaderInfoLog, PFNGLGETSHADERINFOLOGPROC),
            glGetShaderPrecisionFormat: get_fn!(
                glGetShaderPrecisionFormat,
                PFNGLGETSHADERPRECISIONFORMATPROC
            ),
            glGetShaderSource: get_fn!(glGetShaderSource, PFNGLGETSHADERSOURCEPROC),
            glGetString: get_fn!(glGetString, PFNGLGETSTRINGPROC),
            glGetTexParameterfv: get_fn!(glGetTexParameterfv, PFNGLGETTEXPARAMETERFVPROC),
            glGetTexParameteriv: get_fn!(glGetTexParameteriv, PFNGLGETTEXPARAMETERIVPROC),
            glGetUniformfv: get_fn!(glGetUniformfv, PFNGLGETUNIFORMFVPROC),
            glGetUniformiv: get_fn!(glGetUniformiv, PFNGLGETUNIFORMIVPROC),
            glGetUniformLocation: get_fn!(glGetUniformLocation, PFNGLGETUNIFORMLOCATIONPROC),
            glGetVertexAttribfv: get_fn!(glGetVertexAttribfv, PFNGLGETVERTEXATTRIBFVPROC),
            glGetVertexAttribiv: get_fn!(glGetVertexAttribiv, PFNGLGETVERTEXATTRIBIVPROC),
            glGetVertexAttribPointerv: get_fn!(
                glGetVertexAttribPointerv,
                PFNGLGETVERTEXATTRIBPOINTERVPROC
            ),
            glHint: get_fn!(glHint, PFNGLHINTPROC),
            glIsBuffer: get_fn!(glIsBuffer, PFNGLISBUFFERPROC),
            glIsEnabled: get_fn!(glIsEnabled, PFNGLISENABLEDPROC),
            glIsFramebuffer: get_fn!(glIsFramebuffer, PFNGLISFRAMEBUFFERPROC),
            glIsProgram: get_fn!(glIsProgram, PFNGLISPROGRAMPROC),
            glIsRenderbuffer: get_fn!(glIsRenderbuffer, PFNGLISRENDERBUFFERPROC),
            glIsShader: get_fn!(glIsShader, PFNGLISSHADERPROC),
            glIsTexture: get_fn!(glIsTexture, PFNGLISTEXTUREPROC),
            glLineWidth: get_fn!(glLineWidth, PFNGLLINEWIDTHPROC),
            glLinkProgram: get_fn!(glLinkProgram, PFNGLLINKPROGRAMPROC),
            glPixelStorei: get_fn!(glPixelStorei, PFNGLPIXELSTOREIPROC),
            glPolygonOffset: get_fn!(glPolygonOffset, PFNGLPOLYGONOFFSETPROC),
            glReadPixels: get_fn!(glReadPixels, PFNGLREADPIXELSPROC),
            glReleaseShaderCompiler: get_fn!(
                glReleaseShaderCompiler,
                PFNGLRELEASESHADERCOMPILERPROC
            ),
            glRenderbufferStorage: get_fn!(glRenderbufferStorage, PFNGLRENDERBUFFERSTORAGEPROC),
            glSampleCoverage: get_fn!(glSampleCoverage, PFNGLSAMPLECOVERAGEPROC),
            glScissor: get_fn!(glScissor, PFNGLSCISSORPROC),
            glShaderBinary: get_fn!(glShaderBinary, PFNGLSHADERBINARYPROC),
            glShaderSource: get_fn!(glShaderSource, PFNGLSHADERSOURCEPROC),
            glStencilFunc: get_fn!(glStencilFunc, PFNGLSTENCILFUNCPROC),
            glStencilFuncSeparate: get_fn!(glStencilFuncSeparate, PFNGLSTENCILFUNCSEPARATEPROC),
            glStencilMask: get_fn!(glStencilMask, PFNGLSTENCILMASKPROC),
            glStencilMaskSeparate: get_fn!(glStencilMaskSeparate, PFNGLSTENCILMASKSEPARATEPROC),
            glStencilOp: get_fn!(glStencilOp, PFNGLSTENCILOPPROC),
            glStencilOpSeparate: get_fn!(glStencilOpSeparate, PFNGLSTENCILOPSEPARATEPROC),
            glTexImage2D: get_fn!(glTexImage2D, PFNGLTEXIMAGE2DPROC),
            glTexParameterf: get_fn!(glTexParameterf, PFNGLTEXPARAMETERFPROC),
            glTexParameterfv: get_fn!(glTexParameterfv, PFNGLTEXPARAMETERFVPROC),
            glTexParameteri: get_fn!(glTexParameteri, PFNGLTEXPARAMETERIPROC),
            glTexParameteriv: get_fn!(glTexParameteriv, PFNGLTEXPARAMETERIVPROC),
            glTexSubImage2D: get_fn!(glTexSubImage2D, PFNGLTEXSUBIMAGE2DPROC),
            glUniform1f: get_fn!(glUniform1f, PFNGLUNIFORM1FPROC),
            glUniform1fv: get_fn!(glUniform1fv, PFNGLUNIFORM1FVPROC),
            glUniform1i: get_fn!(glUniform1i, PFNGLUNIFORM1IPROC),
            glUniform1iv: get_fn!(glUniform1iv, PFNGLUNIFORM1IVPROC),
            glUniform2f: get_fn!(glUniform2f, PFNGLUNIFORM2FPROC),
            glUniform2fv: get_fn!(glUniform2fv, PFNGLUNIFORM2FVPROC),
            glUniform2i: get_fn!(glUniform2i, PFNGLUNIFORM2IPROC),
            glUniform2iv: get_fn!(glUniform2iv, PFNGLUNIFORM2IVPROC),
            glUniform3f: get_fn!(glUniform3f, PFNGLUNIFORM3FPROC),
            glUniform3fv: get_fn!(glUniform3fv, PFNGLUNIFORM3FVPROC),
            glUniform3i: get_fn!(glUniform3i, PFNGLUNIFORM3IPROC),
            glUniform3iv: get_fn!(glUniform3iv, PFNGLUNIFORM3IVPROC),
            glUniform4f: get_fn!(glUniform4f, PFNGLUNIFORM4FPROC),
            glUniform4fv: get_fn!(glUniform4fv, PFNGLUNIFORM4FVPROC),
            glUniform4i: get_fn!(glUniform4i, PFNGLUNIFORM4IPROC),
            glUniform4iv: get_fn!(glUniform4iv, PFNGLUNIFORM4IVPROC),
            glUniformMatrix2fv: get_fn!(glUniformMatrix2fv, PFNGLUNIFORMMATRIX2FVPROC),
            glUniformMatrix3fv: get_fn!(glUniformMatrix3fv, PFNGLUNIFORMMATRIX3FVPROC),
            glUniformMatrix4fv: get_fn!(glUniformMatrix4fv, PFNGLUNIFORMMATRIX4FVPROC),
            glUseProgram: get_fn!(glUseProgram, PFNGLUSEPROGRAMPROC),
            glValidateProgram: get_fn!(glValidateProgram, PFNGLVALIDATEPROGRAMPROC),
            glVertexAttrib1f: get_fn!(glVertexAttrib1f, PFNGLVERTEXATTRIB1FPROC),
            glVertexAttrib1fv: get_fn!(glVertexAttrib1fv, PFNGLVERTEXATTRIB1FVPROC),
            glVertexAttrib2f: get_fn!(glVertexAttrib2f, PFNGLVERTEXATTRIB2FPROC),
            glVertexAttrib2fv: get_fn!(glVertexAttrib2fv, PFNGLVERTEXATTRIB2FVPROC),
            glVertexAttrib3f: get_fn!(glVertexAttrib3f, PFNGLVERTEXATTRIB3FPROC),
            glVertexAttrib3fv: get_fn!(glVertexAttrib3fv, PFNGLVERTEXATTRIB3FVPROC),
            glVertexAttrib4f: get_fn!(glVertexAttrib4f, PFNGLVERTEXATTRIB4FPROC),
            glVertexAttrib4fv: get_fn!(glVertexAttrib4fv, PFNGLVERTEXATTRIB4FVPROC),
            glVertexAttribPointer: get_fn!(glVertexAttribPointer, PFNGLVERTEXATTRIBPOINTERPROC),
            glViewport: get_fn!(glViewport, PFNGLVIEWPORTPROC),
        }
    }
}

#[derive(Debug)]
pub enum GLES2Error {
    SDLError(sdl::SDLError),
    NoShaderCompilerSupport,
}

impl From<sdl::SDLError> for GLES2Error {
    fn from(v: sdl::SDLError) -> Self {
        GLES2Error::SDLError(v)
    }
}

impl fmt::Display for GLES2Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GLES2Error::SDLError(error) => (error as &fmt::Display).fmt(f),
            GLES2Error::NoShaderCompilerSupport => {
                f.write_str("the OpenGL ES implementation doesn't support compiling shaders")
            }
        }
    }
}

impl error::Error for GLES2Error {}

pub struct GLES2StagingVertexBuffer {
    buffer_data: Vec<VertexBufferElement>,
}

impl GLES2StagingVertexBuffer {
    fn new(len: usize) -> Self {
        let mut buffer_data = Vec::new();
        buffer_data.resize(len, unsafe { mem::zeroed() });
        Self {
            buffer_data: buffer_data,
        }
    }
}

impl StagingVertexBuffer for GLES2StagingVertexBuffer {
    fn len(&self) -> usize {
        self.buffer_data.len()
    }
    fn write(&mut self, index: usize, value: VertexBufferElement) {
        self.buffer_data[index] = value;
    }
}

struct DeviceBuffer {
    buffer: api::GLuint,
    buffer_deallocate_channel_sender: mpsc::Sender<api::GLuint>,
}

impl Drop for DeviceBuffer {
    fn drop(&mut self) {
        self.buffer_deallocate_channel_sender
            .send(self.buffer)
            .unwrap_or_default();
    }
}

#[derive(Clone)]
pub struct GLES2DeviceVertexBuffer {
    buffer: Arc<Mutex<Option<DeviceBuffer>>>,
    len: usize,
    load_submitted_flag: Arc<atomic::AtomicBool>,
}

impl DeviceVertexBuffer for GLES2DeviceVertexBuffer {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct GLES2StagingIndexBuffer {
    buffer_data: Vec<IndexBufferElement>,
}

impl GLES2StagingIndexBuffer {
    fn new(len: usize) -> Self {
        let mut buffer_data = Vec::new();
        buffer_data.resize(len, Default::default());
        Self {
            buffer_data: buffer_data,
        }
    }
}

impl StagingIndexBuffer for GLES2StagingIndexBuffer {
    fn len(&self) -> usize {
        self.buffer_data.len()
    }
    fn write(&mut self, index: usize, value: IndexBufferElement) {
        self.buffer_data[index] = value;
    }
}

#[derive(Clone)]
pub struct GLES2DeviceIndexBuffer {
    buffer: Arc<Mutex<Option<DeviceBuffer>>>,
    len: usize,
    load_submitted_flag: Arc<atomic::AtomicBool>,
}

impl DeviceIndexBuffer for GLES2DeviceIndexBuffer {
    fn len(&self) -> usize {
        self.len
    }
}

#[derive(Clone)]
pub struct GLES2DeviceReference {}

enum LoaderCommand {
    CopyVertexBufferToDevice {
        staging_vertex_buffer: GLES2StagingVertexBuffer,
        device_vertex_buffer: GLES2DeviceVertexBuffer,
    },
    CopyIndexBufferToDevice {
        staging_index_buffer: GLES2StagingIndexBuffer,
        device_index_buffer: GLES2DeviceIndexBuffer,
    },
}

pub struct GLES2LoaderCommandBuffer {
    commands: Vec<LoaderCommand>,
    submitted_flag: Arc<atomic::AtomicBool>,
}

impl CommandBuffer for GLES2LoaderCommandBuffer {}

pub struct GLES2LoaderCommandBufferBuilder {
    command_buffer: GLES2LoaderCommandBuffer,
}

impl GLES2LoaderCommandBufferBuilder {
    fn new() -> Self {
        Self {
            command_buffer: GLES2LoaderCommandBuffer {
                commands: Vec::new(),
                submitted_flag: Arc::new(atomic::AtomicBool::new(false)),
            },
        }
    }
}

impl LoaderCommandBufferBuilder for GLES2LoaderCommandBufferBuilder {
    type Error = GLES2Error;
    type CommandBuffer = GLES2LoaderCommandBuffer;
    type StagingVertexBuffer = GLES2StagingVertexBuffer;
    type DeviceVertexBuffer = GLES2DeviceVertexBuffer;
    type StagingIndexBuffer = GLES2StagingIndexBuffer;
    type DeviceIndexBuffer = GLES2DeviceIndexBuffer;
    fn finish(self) -> Result<GLES2LoaderCommandBuffer> {
        Ok(self.command_buffer)
    }
    fn copy_vertex_buffer_to_device(
        &mut self,
        staging_vertex_buffer: GLES2StagingVertexBuffer,
    ) -> Result<GLES2DeviceVertexBuffer> {
        let device_buffer = GLES2DeviceVertexBuffer {
            buffer: Arc::new(Mutex::new(None)),
            len: staging_vertex_buffer.buffer_data.len(),
            load_submitted_flag: self.command_buffer.submitted_flag.clone(),
        };
        self.command_buffer
            .commands
            .push(LoaderCommand::CopyVertexBufferToDevice {
                staging_vertex_buffer: staging_vertex_buffer,
                device_vertex_buffer: device_buffer.clone(),
            });
        Ok(device_buffer)
    }
    fn copy_index_buffer_to_device(
        &mut self,
        staging_index_buffer: GLES2StagingIndexBuffer,
    ) -> Result<GLES2DeviceIndexBuffer> {
        let device_buffer = GLES2DeviceIndexBuffer {
            buffer: Arc::new(Mutex::new(None)),
            len: staging_index_buffer.buffer_data.len(),
            load_submitted_flag: self.command_buffer.submitted_flag.clone(),
        };
        self.command_buffer
            .commands
            .push(LoaderCommand::CopyIndexBufferToDevice {
                staging_index_buffer: staging_index_buffer,
                device_index_buffer: device_buffer.clone(),
            });
        Ok(device_buffer)
    }
}

enum RenderCommand {
    Draw {
        vertex_buffer: GLES2DeviceVertexBuffer,
        index_buffer: GLES2DeviceIndexBuffer,
        initial_transform: math::Mat4<f32>,
        index_count: u32,
        first_index: u32,
        vertex_offset: u32,
    },
}

struct RenderCommandBufferState {
    commands: Vec<RenderCommand>,
}

#[derive(Clone)]
pub struct GLES2RenderCommandBuffer(Arc<RenderCommandBufferState>);

impl CommandBuffer for GLES2RenderCommandBuffer {}

pub struct GLES2RenderCommandBufferBuilder {
    state: RenderCommandBufferState,
    buffers: Option<(GLES2DeviceVertexBuffer, GLES2DeviceIndexBuffer)>,
    initial_transform: math::Mat4<f32>,
}

impl GLES2RenderCommandBufferBuilder {
    fn new() -> Self {
        Self {
            state: RenderCommandBufferState {
                commands: Vec::new(),
            },
            buffers: None,
            initial_transform: math::Mat4::identity(),
        }
    }
}

impl RenderCommandBufferBuilder for GLES2RenderCommandBufferBuilder {
    type Error = GLES2Error;
    type CommandBuffer = GLES2RenderCommandBuffer;
    type DeviceVertexBuffer = GLES2DeviceVertexBuffer;
    type DeviceIndexBuffer = GLES2DeviceIndexBuffer;
    fn set_buffers(
        &mut self,
        vertex_buffer: GLES2DeviceVertexBuffer,
        index_buffer: GLES2DeviceIndexBuffer,
    ) {
        self.buffers = Some((vertex_buffer, index_buffer));
    }
    fn set_initial_transform(&mut self, transform: math::Mat4<f32>) {
        self.initial_transform = transform;
    }
    fn draw(&mut self, index_count: u32, first_index: u32, vertex_offset: u32) {
        let (vertex_buffer, index_buffer) = self
            .buffers
            .clone()
            .expect("can't draw without vertex and index buffers bound");
        assert!(index_count as usize <= index_buffer.len());
        assert!(index_count as usize + first_index as usize <= index_buffer.len());
        assert!((vertex_offset as usize) < vertex_buffer.len());
        assert!(index_count % 3 == 0, "must be whole number of triangles");
        if index_count == 0 {
            return;
        }
        self.state.commands.push(RenderCommand::Draw {
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            initial_transform: self.initial_transform,
            index_count: index_count,
            first_index: first_index,
            vertex_offset: vertex_offset,
        });
    }
    fn finish(self) -> Result<GLES2RenderCommandBuffer> {
        Ok(GLES2RenderCommandBuffer(Arc::new(self.state)))
    }
}

impl DeviceReference for GLES2DeviceReference {
    type Error = GLES2Error;
    type LoaderCommandBuffer = GLES2LoaderCommandBuffer;
    type LoaderCommandBufferBuilder = GLES2LoaderCommandBufferBuilder;
    type RenderCommandBuffer = GLES2RenderCommandBuffer;
    type RenderCommandBufferBuilder = GLES2RenderCommandBufferBuilder;
    type StagingVertexBuffer = GLES2StagingVertexBuffer;
    type DeviceVertexBuffer = GLES2DeviceVertexBuffer;
    type StagingIndexBuffer = GLES2StagingIndexBuffer;
    type DeviceIndexBuffer = GLES2DeviceIndexBuffer;
    fn create_loader_command_buffer_builder(&self) -> Result<GLES2LoaderCommandBufferBuilder> {
        Ok(GLES2LoaderCommandBufferBuilder::new())
    }
    fn create_render_command_buffer_builder(&self) -> Result<GLES2RenderCommandBufferBuilder> {
        Ok(GLES2RenderCommandBufferBuilder::new())
    }
    fn create_staging_vertex_buffer(&self, len: usize) -> Result<GLES2StagingVertexBuffer> {
        Ok(GLES2StagingVertexBuffer::new(len))
    }
    fn create_staging_index_buffer(&self, len: usize) -> Result<GLES2StagingIndexBuffer> {
        Ok(GLES2StagingIndexBuffer::new(len))
    }
}

struct SurfaceState {
    window: sdl::window::Window,
}

pub struct GLES2PausedDevice {
    surface_state: SurfaceState,
}

struct GLContextWrapper {
    context: sdl::api::SDL_GLContext,
    api: Api,
}

impl GLContextWrapper {
    unsafe fn new(window: &sdl::window::Window) -> Result<Self> {
        let mut temp_context = TempContextWrapper {
            context: sdl::api::SDL_GL_CreateContext(window.get()),
        };
        if temp_context.context.is_null() {
            return Err(GLES2Error::SDLError(sdl::get_error()));
        }
        let api = Api::new();
        return Ok(Self {
            context: mem::replace(&mut temp_context.context, null_mut()),
            api: api,
        });
        struct TempContextWrapper {
            context: sdl::api::SDL_GLContext,
        }
        impl Drop for TempContextWrapper {
            fn drop(&mut self) {
                if self.context.is_null() {
                    return;
                }
                unsafe {
                    sdl::api::SDL_GL_DeleteContext(self.context);
                }
            }
        }
    }
}

impl Drop for GLContextWrapper {
    fn drop(&mut self) {
        unsafe {
            sdl::api::SDL_GL_DeleteContext(self.context);
        }
    }
}

#[derive(Clone, Copy)]
struct ShaderAttributeLocations {
    input_position: api::GLint,
    input_color: api::GLint,
    input_texture_coord: api::GLint,
    input_texture_index: api::GLint,
}

struct ShaderUniformLocations {
    initial_transform: api::GLint,
    final_transform: api::GLint,
}

pub struct GLES2Device {
    device_reference: GLES2DeviceReference,
    surface_state: SurfaceState,
    gl_context: GLContextWrapper,
    buffer_deallocate_channel_sender: mpsc::Sender<api::GLuint>,
    buffer_deallocate_channel_receiver: mpsc::Receiver<api::GLuint>,
    shader_attribute_locations: ShaderAttributeLocations,
    shader_uniform_locations: ShaderUniformLocations,
    last_surface_dimensions: Option<(u32, u32)>,
}

impl GLES2Device {
    fn allocate_buffer(&mut self) -> DeviceBuffer {
        match self.buffer_deallocate_channel_receiver.try_recv() {
            Ok(buffer) => DeviceBuffer {
                buffer: buffer,
                buffer_deallocate_channel_sender: self.buffer_deallocate_channel_sender.clone(),
            },
            Err(mpsc::TryRecvError::Empty) => unsafe {
                let api = &self.gl_context.api;
                let mut buffer = 0;
                api.glGenBuffers.unwrap()(1, &mut buffer);
                DeviceBuffer {
                    buffer: buffer,
                    buffer_deallocate_channel_sender: self.buffer_deallocate_channel_sender.clone(),
                }
            },
            _ => panic!(),
        }
    }
}

pub type Result<T> = result::Result<T, GLES2Error>;

impl PausedDevice for GLES2PausedDevice {
    type Device = GLES2Device;
    fn get_window(&self) -> &sdl::window::Window {
        &self.surface_state.window
    }
}

impl Device for GLES2Device {
    type Error = GLES2Error;
    type Reference = GLES2DeviceReference;
    type PausedDevice = GLES2PausedDevice;
    type LoaderCommandBuffer = GLES2LoaderCommandBuffer;
    type LoaderCommandBufferBuilder = GLES2LoaderCommandBufferBuilder;
    type RenderCommandBuffer = GLES2RenderCommandBuffer;
    type RenderCommandBufferBuilder = GLES2RenderCommandBufferBuilder;
    type StagingVertexBuffer = GLES2StagingVertexBuffer;
    type DeviceVertexBuffer = GLES2DeviceVertexBuffer;
    type StagingIndexBuffer = GLES2StagingIndexBuffer;
    type DeviceIndexBuffer = GLES2DeviceIndexBuffer;
    fn pause(self) -> GLES2PausedDevice {
        GLES2PausedDevice {
            surface_state: self.surface_state,
        }
    }
    fn resume(paused_device: GLES2PausedDevice) -> Result<Self> {
        let vertex_shader_source: &'static CStr = CStr::from_bytes_with_nul(
            concat!(
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/shaders/gles2_main.vert"
                )),
                "\0"
            ).as_bytes(),
        ).unwrap();
        let fragment_shader_source: &'static CStr = CStr::from_bytes_with_nul(
            concat!(
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/shaders/gles2_main.frag"
                )),
                "\0"
            ).as_bytes(),
        ).unwrap();
        unsafe {
            let SurfaceState { window } = paused_device.surface_state;
            set_gl_attributes()?;
            let gl_context = GLContextWrapper::new(&window)?;
            if sdl::api::SDL_GL_SetSwapInterval(0) != 0 {
                eprintln!("can't set swap interval: {}", sdl::get_error());
            }
            let shader_attribute_locations;
            let shader_uniform_locations;
            {
                let api = &gl_context.api;
                api.glEnable.unwrap()(api::GL_BLEND);
                api.glEnable.unwrap()(api::GL_CULL_FACE);
                api.glEnable.unwrap()(api::GL_DEPTH_TEST);
                api.glBlendFunc.unwrap()(api::GL_SRC_ALPHA, api::GL_ONE_MINUS_SRC_ALPHA);
                let mut shader_compiler_supported = api::GL_FALSE as api::GLboolean;
                api.glGetBooleanv.unwrap()(api::GL_SHADER_COMPILER, &mut shader_compiler_supported);
                if shader_compiler_supported == api::GL_FALSE as api::GLboolean {
                    return Err(GLES2Error::NoShaderCompilerSupport);
                }
                let vertex_shader = api.glCreateShader.unwrap()(api::GL_VERTEX_SHADER);
                assert_ne!(vertex_shader, 0);
                api.glShaderSource.unwrap()(
                    vertex_shader,
                    1,
                    &vertex_shader_source.as_ptr(),
                    null(),
                );
                api.glCompileShader.unwrap()(vertex_shader);
                fn is_info_log_empty(info_log: &str) -> bool {
                    for c in info_log.chars() {
                        match c {
                            ' ' | '\r' | '\n' | '\t' => {}
                            _ => {
                                return false;
                            }
                        }
                    }
                    true
                }
                let get_shader_info_log = |shader: api::GLuint| -> CString {
                    let mut length = 0;
                    api.glGetShaderiv.unwrap()(shader, api::GL_INFO_LOG_LENGTH, &mut length);
                    let mut buffer = Vec::new();
                    buffer.resize(length as usize, 0);
                    let mut length = 0;
                    api.glGetShaderInfoLog.unwrap()(
                        shader,
                        buffer.len() as api::GLsizei,
                        &mut length,
                        buffer.as_mut_ptr() as *mut c_char,
                    );
                    buffer.resize(length as usize + 1, 0);
                    CString::from_vec_unchecked(buffer)
                };
                let write_shader_info_log = |shader: api::GLuint, name: &str| {
                    let info_log = get_shader_info_log(shader);
                    let info_log = info_log.to_string_lossy();
                    if !is_info_log_empty(&info_log) {
                        println!("{}:\n{}", name, info_log);
                    }
                };
                let get_program_info_log = |program: api::GLuint| -> CString {
                    let mut length = 0;
                    api.glGetProgramiv.unwrap()(program, api::GL_INFO_LOG_LENGTH, &mut length);
                    let mut buffer = Vec::new();
                    buffer.resize(length as usize, 0);
                    let mut length = 0;
                    api.glGetProgramInfoLog.unwrap()(
                        program,
                        buffer.len() as api::GLsizei,
                        &mut length,
                        buffer.as_mut_ptr() as *mut c_char,
                    );
                    buffer.resize(length as usize + 1, 0);
                    CString::from_vec_unchecked(buffer)
                };
                let write_program_info_log = |program: api::GLuint| {
                    let info_log = get_program_info_log(program);
                    let info_log = info_log.to_string_lossy();
                    if !is_info_log_empty(&info_log) {
                        println!("Program Link:\n{}", info_log);
                    }
                };
                let get_shader_compile_status = |shader: api::GLuint| {
                    let mut compile_status = api::GL_FALSE as api::GLint;
                    api.glGetShaderiv.unwrap()(shader, api::GL_COMPILE_STATUS, &mut compile_status);
                    compile_status != api::GL_FALSE as api::GLint
                };
                let get_program_link_status = |program: api::GLuint| {
                    let mut link_status = api::GL_FALSE as api::GLint;
                    api.glGetProgramiv.unwrap()(program, api::GL_LINK_STATUS, &mut link_status);
                    link_status != api::GL_FALSE as api::GLint
                };
                write_shader_info_log(vertex_shader, "Vertex Shader");
                assert!(
                    get_shader_compile_status(vertex_shader),
                    "vertex shader compile failed"
                );
                let fragment_shader = api.glCreateShader.unwrap()(api::GL_FRAGMENT_SHADER);
                assert_ne!(fragment_shader, 0);
                api.glShaderSource.unwrap()(
                    fragment_shader,
                    1,
                    &fragment_shader_source.as_ptr(),
                    null(),
                );
                api.glCompileShader.unwrap()(fragment_shader);
                write_shader_info_log(fragment_shader, "Fragment Shader");
                assert!(
                    get_shader_compile_status(fragment_shader),
                    "fragment shader compile failed"
                );
                let shader_program = api.glCreateProgram.unwrap()();
                assert_ne!(shader_program, 0);
                api.glAttachShader.unwrap()(shader_program, vertex_shader);
                api.glAttachShader.unwrap()(shader_program, fragment_shader);
                api.glLinkProgram.unwrap()(shader_program);
                write_program_info_log(shader_program);
                assert!(
                    get_program_link_status(shader_program),
                    "program link failed"
                );
                api.glUseProgram.unwrap()(shader_program);
                macro_rules! shader_attribute_locations {
                    ($program:expr, ($($name:ident,)*)) => {
                        ShaderAttributeLocations {
                            $(
                                $name: {
                                    let location = api.glGetAttribLocation.unwrap()(
                                        $program,
                                        concat!(stringify!($name), "\0").as_ptr() as *const c_char,
                                    );
                                    if location != -1 {
                                        api.glEnableVertexAttribArray.unwrap()(location as api::GLuint);
                                    }
                                    location
                                },
                            )*
                        }
                    };
                }
                macro_rules! shader_uniform_locations {
                    ($program:expr, ($($name:ident,)*)) => {
                        ShaderUniformLocations {
                            $(
                                $name: api.glGetUniformLocation.unwrap()(
                                    $program,
                                    concat!(stringify!($name), "\0").as_ptr() as *const c_char,
                                ),
                            )*
                        }
                    };
                }
                shader_attribute_locations = shader_attribute_locations!(
                    shader_program,
                    (
                        input_position,
                        input_color,
                        input_texture_coord,
                        input_texture_index,
                    )
                );
                shader_uniform_locations = shader_uniform_locations!(
                    shader_program,
                    (initial_transform, final_transform,)
                );
            }
            let (buffer_deallocate_channel_sender, buffer_deallocate_channel_receiver) =
                mpsc::channel();
            Ok(GLES2Device {
                device_reference: GLES2DeviceReference {},
                surface_state: SurfaceState { window: window },
                gl_context: gl_context,
                buffer_deallocate_channel_sender: buffer_deallocate_channel_sender,
                buffer_deallocate_channel_receiver: buffer_deallocate_channel_receiver,
                shader_attribute_locations: shader_attribute_locations,
                shader_uniform_locations: shader_uniform_locations,
                last_surface_dimensions: None,
            })
        }
    }
    fn get_device_ref(&self) -> &GLES2DeviceReference {
        &self.device_reference
    }
    fn get_window(&self) -> &sdl::window::Window {
        &self.surface_state.window
    }
    fn submit_loader_command_buffers(
        &mut self,
        loader_command_buffers: &mut Vec<GLES2LoaderCommandBuffer>,
    ) -> Result<()> {
        for loader_command_buffer in loader_command_buffers.drain(..) {
            loader_command_buffer
                .submitted_flag
                .store(true, atomic::Ordering::Release);
            for command in loader_command_buffer.commands {
                match command {
                    LoaderCommand::CopyVertexBufferToDevice {
                        staging_vertex_buffer: GLES2StagingVertexBuffer { buffer_data },
                        device_vertex_buffer:
                            GLES2DeviceVertexBuffer {
                                buffer,
                                len: _,
                                load_submitted_flag: _,
                            },
                    } => {
                        let new_buffer = self.allocate_buffer();
                        unsafe {
                            let api = &self.gl_context.api;
                            api.glBindBuffer.unwrap()(api::GL_ARRAY_BUFFER, new_buffer.buffer);
                            api.glBufferData.unwrap()(
                                api::GL_ARRAY_BUFFER,
                                (buffer_data.len() * mem::size_of::<VertexBufferElement>())
                                    as api::GLsizeiptr,
                                buffer_data.as_ptr() as *const c_void,
                                api::GL_STATIC_DRAW,
                            );
                        }
                        *buffer.lock().unwrap() = Some(new_buffer);
                    }
                    LoaderCommand::CopyIndexBufferToDevice {
                        staging_index_buffer: GLES2StagingIndexBuffer { buffer_data },
                        device_index_buffer:
                            GLES2DeviceIndexBuffer {
                                buffer,
                                len: _,
                                load_submitted_flag: _,
                            },
                    } => {
                        let new_buffer = self.allocate_buffer();
                        unsafe {
                            let api = &self.gl_context.api;
                            api.glBindBuffer.unwrap()(
                                api::GL_ELEMENT_ARRAY_BUFFER,
                                new_buffer.buffer,
                            );
                            api.glBufferData.unwrap()(
                                api::GL_ELEMENT_ARRAY_BUFFER,
                                (buffer_data.len() * mem::size_of::<IndexBufferElement>())
                                    as api::GLsizeiptr,
                                buffer_data.as_ptr() as *const c_void,
                                api::GL_STATIC_DRAW,
                            );
                        }
                        *buffer.lock().unwrap() = Some(new_buffer);
                    }
                }
            }
        }
        Ok(())
    }
    fn render_frame(
        &mut self,
        clear_color: math::Vec4<f32>,
        loader_command_buffers: &mut Vec<GLES2LoaderCommandBuffer>,
        render_command_buffer_groups: &[RenderCommandBufferGroup<GLES2RenderCommandBuffer>],
    ) -> Result<()> {
        unsafe {
            let mut sdl_dimensions = (0, 0);
            sdl::api::SDL_GL_GetDrawableSize(
                self.surface_state.window.get(),
                &mut sdl_dimensions.0,
                &mut sdl_dimensions.1,
            );
            let sdl_dimensions = (sdl_dimensions.0 as u32, sdl_dimensions.1 as u32);
            if Some(sdl_dimensions) != self.last_surface_dimensions {
                self.last_surface_dimensions = Some(sdl_dimensions);
                let api = &self.gl_context.api;
                api.glViewport.unwrap()(
                    0,
                    0,
                    sdl_dimensions.0 as api::GLsizei,
                    sdl_dimensions.1 as api::GLsizei,
                );
            }
            self.submit_loader_command_buffers(loader_command_buffers)?;
            let api = &self.gl_context.api;
            api.glClearColor.unwrap()(clear_color.x, clear_color.y, clear_color.z, clear_color.w);
            api.glClear.unwrap()(api::GL_COLOR_BUFFER_BIT | api::GL_DEPTH_BUFFER_BIT);
            for render_command_buffer_group in render_command_buffer_groups {
                let set_uniform_matrix = |location: api::GLint, value: math::Mat4<f32>| {
                    if location != -1 {
                        let value: [[f32; 4]; 4] = value.into();
                        api.glUniformMatrix4fv.unwrap()(
                            location,
                            1,
                            api::GL_FALSE as api::GLboolean,
                            &value as *const [f32; 4] as *const f32,
                        )
                    }
                };
                set_uniform_matrix(
                    self.shader_uniform_locations.final_transform,
                    render_command_buffer_group.final_transform,
                );
                for GLES2RenderCommandBuffer(state) in
                    render_command_buffer_group.render_command_buffers
                {
                    for command in &state.commands {
                        match command {
                            RenderCommand::Draw {
                                vertex_buffer:
                                    GLES2DeviceVertexBuffer {
                                        buffer: vertex_buffer,
                                        len: _,
                                        load_submitted_flag: vertex_buffer_load_submitted_flag,
                                    },
                                index_buffer:
                                    GLES2DeviceIndexBuffer {
                                        buffer: index_buffer,
                                        len: _,
                                        load_submitted_flag: index_buffer_load_submitted_flag,
                                    },
                                initial_transform,
                                index_count,
                                first_index,
                                vertex_offset,
                            } => {
                                assert!(
                                    vertex_buffer_load_submitted_flag
                                        .load(atomic::Ordering::Acquire)
                                );
                                assert!(
                                    index_buffer_load_submitted_flag
                                        .load(atomic::Ordering::Acquire)
                                );
                                set_uniform_matrix(
                                    self.shader_uniform_locations.initial_transform,
                                    *initial_transform,
                                );
                                api.glBindBuffer.unwrap()(
                                    api::GL_ELEMENT_ARRAY_BUFFER,
                                    index_buffer.lock().unwrap().as_ref().unwrap().buffer,
                                );
                                api.glBindBuffer.unwrap()(
                                    api::GL_ARRAY_BUFFER,
                                    vertex_buffer.lock().unwrap().as_ref().unwrap().buffer,
                                );
                                macro_rules! set_attributes {
                                    ($(($name:ident, $member:ident, $size:expr, $type:expr, $normalized:expr),)*) => {
                                        {
                                            let ShaderAttributeLocations{$($name,)*} = self.shader_attribute_locations;
                                            $(
                                                if $name != -1 {
                                                    let vertex: VertexBufferElement = mem::uninitialized();
                                                    let offset = &vertex.$member as *const _ as usize - &vertex as *const _ as usize;
                                                    mem::forget(vertex);
                                                    api.glVertexAttribPointer.unwrap()(
                                                        $name as api::GLuint,
                                                        $size,
                                                        $type,
                                                        $normalized as api::GLboolean,
                                                        mem::size_of::<VertexBufferElement>() as api::GLsizei,
                                                        (mem::size_of::<VertexBufferElement>() * *vertex_offset as usize + offset) as *const _,
                                                    );
                                                }
                                            )*
                                        }
                                    };
                                }
                                set_attributes!(
                                    (input_position, position, 3, api::GL_FLOAT, api::GL_FALSE),
                                    (input_color, color, 4, api::GL_UNSIGNED_BYTE, api::GL_TRUE),
                                    (
                                        input_texture_coord,
                                        texture_coord,
                                        2,
                                        api::GL_FLOAT,
                                        api::GL_FALSE
                                    ),
                                    (
                                        input_texture_index,
                                        texture_index,
                                        1,
                                        api::GL_UNSIGNED_SHORT,
                                        api::GL_FALSE
                                    ),
                                );
                                api.glDrawElements.unwrap()(
                                    api::GL_TRIANGLES,
                                    *index_count as api::GLsizei,
                                    api::GL_UNSIGNED_SHORT,
                                    (*first_index as usize * mem::size_of::<IndexBufferElement>())
                                        as *const _,
                                );
                            }
                        }
                    }
                }
            }
            sdl::api::SDL_GL_SwapWindow(self.surface_state.window.get());
            Ok(())
        }
    }
}

pub struct GLES2DeviceFactory<'a>(&'a sdl::event::EventSource);

impl<'a> GLES2DeviceFactory<'a> {
    pub fn new(event_source: &'a sdl::event::EventSource) -> Self {
        GLES2DeviceFactory(event_source)
    }
}

unsafe fn set_gl_attributes() -> Result<()> {
    macro_rules! sdl_gl_set_attribute {
        ($which:ident, $value:expr) => {
            if sdl::api::SDL_GL_SetAttribute(sdl::api::$which, $value) != 0 {
                return Err(sdl::get_error().into());
            }
        };
    }
    sdl::api::SDL_GL_ResetAttributes();
    sdl_gl_set_attribute!(
        SDL_GL_CONTEXT_PROFILE_MASK,
        sdl::api::SDL_GL_CONTEXT_PROFILE_ES as c_int
    );
    sdl_gl_set_attribute!(SDL_GL_CONTEXT_MAJOR_VERSION, 2);
    sdl_gl_set_attribute!(SDL_GL_CONTEXT_MINOR_VERSION, 0);
    Ok(())
}

impl<'a> DeviceFactory for GLES2DeviceFactory<'a> {
    type Device = GLES2Device;
    type Error = GLES2Error;
    type PausedDevice = GLES2PausedDevice;
    fn create<T: Into<String>>(
        &self,
        title: T,
        position: Option<(i32, i32)>,
        size: (u32, u32),
        mut flags: u32,
    ) -> Result<GLES2PausedDevice> {
        assert_eq!(
            flags & (sdl::api::SDL_WINDOW_OPENGL | sdl::api::SDL_WINDOW_VULKAN),
            0
        );
        flags |= sdl::api::SDL_WINDOW_OPENGL;
        if unsafe { sdl::api::SDL_GL_LoadLibrary(null()) } != 0 {
            return Err(sdl::get_error().into());
        }
        unsafe {
            set_gl_attributes()?;
        }
        let window = sdl::window::Window::new(title, position, size, flags)?;
        Ok(GLES2PausedDevice {
            surface_state: SurfaceState { window: window },
        })
    }
}
