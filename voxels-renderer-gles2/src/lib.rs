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
extern crate voxels_image as image;
extern crate voxels_math as math;
extern crate voxels_renderer_base as renderer;
extern crate voxels_sdl as sdl;
use image::Image;
use renderer::*;
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
struct Api {
    glActiveTexture: api::PFNGLACTIVETEXTUREPROC,
    glAttachShader: api::PFNGLATTACHSHADERPROC,
    glBindAttribLocation: api::PFNGLBINDATTRIBLOCATIONPROC,
    glBindBuffer: api::PFNGLBINDBUFFERPROC,
    glBindFramebuffer: api::PFNGLBINDFRAMEBUFFERPROC,
    glBindRenderbuffer: api::PFNGLBINDRENDERBUFFERPROC,
    glBindTexture: api::PFNGLBINDTEXTUREPROC,
    glBlendColor: api::PFNGLBLENDCOLORPROC,
    glBlendEquation: api::PFNGLBLENDEQUATIONPROC,
    glBlendEquationSeparate: api::PFNGLBLENDEQUATIONSEPARATEPROC,
    glBlendFunc: api::PFNGLBLENDFUNCPROC,
    glBlendFuncSeparate: api::PFNGLBLENDFUNCSEPARATEPROC,
    glBufferData: api::PFNGLBUFFERDATAPROC,
    glBufferSubData: api::PFNGLBUFFERSUBDATAPROC,
    glCheckFramebufferStatus: api::PFNGLCHECKFRAMEBUFFERSTATUSPROC,
    glClear: api::PFNGLCLEARPROC,
    glClearColor: api::PFNGLCLEARCOLORPROC,
    glClearDepthf: api::PFNGLCLEARDEPTHFPROC,
    glClearStencil: api::PFNGLCLEARSTENCILPROC,
    glColorMask: api::PFNGLCOLORMASKPROC,
    glCompileShader: api::PFNGLCOMPILESHADERPROC,
    glCompressedTexImage2D: api::PFNGLCOMPRESSEDTEXIMAGE2DPROC,
    glCompressedTexSubImage2D: api::PFNGLCOMPRESSEDTEXSUBIMAGE2DPROC,
    glCopyTexImage2D: api::PFNGLCOPYTEXIMAGE2DPROC,
    glCopyTexSubImage2D: api::PFNGLCOPYTEXSUBIMAGE2DPROC,
    glCreateProgram: api::PFNGLCREATEPROGRAMPROC,
    glCreateShader: api::PFNGLCREATESHADERPROC,
    glCullFace: api::PFNGLCULLFACEPROC,
    glDeleteBuffers: api::PFNGLDELETEBUFFERSPROC,
    glDeleteFramebuffers: api::PFNGLDELETEFRAMEBUFFERSPROC,
    glDeleteProgram: api::PFNGLDELETEPROGRAMPROC,
    glDeleteRenderbuffers: api::PFNGLDELETERENDERBUFFERSPROC,
    glDeleteShader: api::PFNGLDELETESHADERPROC,
    glDeleteTextures: api::PFNGLDELETETEXTURESPROC,
    glDepthFunc: api::PFNGLDEPTHFUNCPROC,
    glDepthMask: api::PFNGLDEPTHMASKPROC,
    glDepthRangef: api::PFNGLDEPTHRANGEFPROC,
    glDetachShader: api::PFNGLDETACHSHADERPROC,
    glDisable: api::PFNGLDISABLEPROC,
    glDisableVertexAttribArray: api::PFNGLDISABLEVERTEXATTRIBARRAYPROC,
    glDrawArrays: api::PFNGLDRAWARRAYSPROC,
    glDrawElements: api::PFNGLDRAWELEMENTSPROC,
    glEnable: api::PFNGLENABLEPROC,
    glEnableVertexAttribArray: api::PFNGLENABLEVERTEXATTRIBARRAYPROC,
    glFinish: api::PFNGLFINISHPROC,
    glFlush: api::PFNGLFLUSHPROC,
    glFramebufferRenderbuffer: api::PFNGLFRAMEBUFFERRENDERBUFFERPROC,
    glFramebufferTexture2D: api::PFNGLFRAMEBUFFERTEXTURE2DPROC,
    glFrontFace: api::PFNGLFRONTFACEPROC,
    glGenBuffers: api::PFNGLGENBUFFERSPROC,
    glGenerateMipmap: api::PFNGLGENERATEMIPMAPPROC,
    glGenFramebuffers: api::PFNGLGENFRAMEBUFFERSPROC,
    glGenRenderbuffers: api::PFNGLGENRENDERBUFFERSPROC,
    glGenTextures: api::PFNGLGENTEXTURESPROC,
    glGetActiveAttrib: api::PFNGLGETACTIVEATTRIBPROC,
    glGetActiveUniform: api::PFNGLGETACTIVEUNIFORMPROC,
    glGetAttachedShaders: api::PFNGLGETATTACHEDSHADERSPROC,
    glGetAttribLocation: api::PFNGLGETATTRIBLOCATIONPROC,
    glGetBooleanv: api::PFNGLGETBOOLEANVPROC,
    glGetBufferParameteriv: api::PFNGLGETBUFFERPARAMETERIVPROC,
    glGetError: api::PFNGLGETERRORPROC,
    glGetFloatv: api::PFNGLGETFLOATVPROC,
    glGetFramebufferAttachmentParameteriv: api::PFNGLGETFRAMEBUFFERATTACHMENTPARAMETERIVPROC,
    glGetIntegerv: api::PFNGLGETINTEGERVPROC,
    glGetProgramiv: api::PFNGLGETPROGRAMIVPROC,
    glGetProgramInfoLog: api::PFNGLGETPROGRAMINFOLOGPROC,
    glGetRenderbufferParameteriv: api::PFNGLGETRENDERBUFFERPARAMETERIVPROC,
    glGetShaderiv: api::PFNGLGETSHADERIVPROC,
    glGetShaderInfoLog: api::PFNGLGETSHADERINFOLOGPROC,
    glGetShaderPrecisionFormat: api::PFNGLGETSHADERPRECISIONFORMATPROC,
    glGetShaderSource: api::PFNGLGETSHADERSOURCEPROC,
    glGetString: api::PFNGLGETSTRINGPROC,
    glGetTexParameterfv: api::PFNGLGETTEXPARAMETERFVPROC,
    glGetTexParameteriv: api::PFNGLGETTEXPARAMETERIVPROC,
    glGetUniformfv: api::PFNGLGETUNIFORMFVPROC,
    glGetUniformiv: api::PFNGLGETUNIFORMIVPROC,
    glGetUniformLocation: api::PFNGLGETUNIFORMLOCATIONPROC,
    glGetVertexAttribfv: api::PFNGLGETVERTEXATTRIBFVPROC,
    glGetVertexAttribiv: api::PFNGLGETVERTEXATTRIBIVPROC,
    glGetVertexAttribPointerv: api::PFNGLGETVERTEXATTRIBPOINTERVPROC,
    glHint: api::PFNGLHINTPROC,
    glIsBuffer: api::PFNGLISBUFFERPROC,
    glIsEnabled: api::PFNGLISENABLEDPROC,
    glIsFramebuffer: api::PFNGLISFRAMEBUFFERPROC,
    glIsProgram: api::PFNGLISPROGRAMPROC,
    glIsRenderbuffer: api::PFNGLISRENDERBUFFERPROC,
    glIsShader: api::PFNGLISSHADERPROC,
    glIsTexture: api::PFNGLISTEXTUREPROC,
    glLineWidth: api::PFNGLLINEWIDTHPROC,
    glLinkProgram: api::PFNGLLINKPROGRAMPROC,
    glPixelStorei: api::PFNGLPIXELSTOREIPROC,
    glPolygonOffset: api::PFNGLPOLYGONOFFSETPROC,
    glReadPixels: api::PFNGLREADPIXELSPROC,
    glReleaseShaderCompiler: api::PFNGLRELEASESHADERCOMPILERPROC,
    glRenderbufferStorage: api::PFNGLRENDERBUFFERSTORAGEPROC,
    glSampleCoverage: api::PFNGLSAMPLECOVERAGEPROC,
    glScissor: api::PFNGLSCISSORPROC,
    glShaderBinary: api::PFNGLSHADERBINARYPROC,
    glShaderSource: api::PFNGLSHADERSOURCEPROC,
    glStencilFunc: api::PFNGLSTENCILFUNCPROC,
    glStencilFuncSeparate: api::PFNGLSTENCILFUNCSEPARATEPROC,
    glStencilMask: api::PFNGLSTENCILMASKPROC,
    glStencilMaskSeparate: api::PFNGLSTENCILMASKSEPARATEPROC,
    glStencilOp: api::PFNGLSTENCILOPPROC,
    glStencilOpSeparate: api::PFNGLSTENCILOPSEPARATEPROC,
    glTexImage2D: api::PFNGLTEXIMAGE2DPROC,
    glTexParameterf: api::PFNGLTEXPARAMETERFPROC,
    glTexParameterfv: api::PFNGLTEXPARAMETERFVPROC,
    glTexParameteri: api::PFNGLTEXPARAMETERIPROC,
    glTexParameteriv: api::PFNGLTEXPARAMETERIVPROC,
    glTexSubImage2D: api::PFNGLTEXSUBIMAGE2DPROC,
    glUniform1f: api::PFNGLUNIFORM1FPROC,
    glUniform1fv: api::PFNGLUNIFORM1FVPROC,
    glUniform1i: api::PFNGLUNIFORM1IPROC,
    glUniform1iv: api::PFNGLUNIFORM1IVPROC,
    glUniform2f: api::PFNGLUNIFORM2FPROC,
    glUniform2fv: api::PFNGLUNIFORM2FVPROC,
    glUniform2i: api::PFNGLUNIFORM2IPROC,
    glUniform2iv: api::PFNGLUNIFORM2IVPROC,
    glUniform3f: api::PFNGLUNIFORM3FPROC,
    glUniform3fv: api::PFNGLUNIFORM3FVPROC,
    glUniform3i: api::PFNGLUNIFORM3IPROC,
    glUniform3iv: api::PFNGLUNIFORM3IVPROC,
    glUniform4f: api::PFNGLUNIFORM4FPROC,
    glUniform4fv: api::PFNGLUNIFORM4FVPROC,
    glUniform4i: api::PFNGLUNIFORM4IPROC,
    glUniform4iv: api::PFNGLUNIFORM4IVPROC,
    glUniformMatrix2fv: api::PFNGLUNIFORMMATRIX2FVPROC,
    glUniformMatrix3fv: api::PFNGLUNIFORMMATRIX3FVPROC,
    glUniformMatrix4fv: api::PFNGLUNIFORMMATRIX4FVPROC,
    glUseProgram: api::PFNGLUSEPROGRAMPROC,
    glValidateProgram: api::PFNGLVALIDATEPROGRAMPROC,
    glVertexAttrib1f: api::PFNGLVERTEXATTRIB1FPROC,
    glVertexAttrib1fv: api::PFNGLVERTEXATTRIB1FVPROC,
    glVertexAttrib2f: api::PFNGLVERTEXATTRIB2FPROC,
    glVertexAttrib2fv: api::PFNGLVERTEXATTRIB2FVPROC,
    glVertexAttrib3f: api::PFNGLVERTEXATTRIB3FPROC,
    glVertexAttrib3fv: api::PFNGLVERTEXATTRIB3FVPROC,
    glVertexAttrib4f: api::PFNGLVERTEXATTRIB4FPROC,
    glVertexAttrib4fv: api::PFNGLVERTEXATTRIB4FVPROC,
    glVertexAttribPointer: api::PFNGLVERTEXATTRIBPOINTERPROC,
    glViewport: api::PFNGLVIEWPORTPROC,
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
    unsafe fn new() -> Self {
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
    ImageIsTooBig,
    ImageMustHavePowerOfTwoDimensions,
    ImageSetHasTooManyImages,
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
            GLES2Error::ImageIsTooBig => f.write_str("image is too big"),
            GLES2Error::ImageMustHavePowerOfTwoDimensions => {
                f.write_str("image must have power-of-two dimensions")
            }
            GLES2Error::ImageSetHasTooManyImages => f.write_str("image set has too many images"),
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

struct DeviceImage {
    image: api::GLuint,
    image_deallocate_channel_sender: mpsc::Sender<api::GLuint>,
}

impl Drop for DeviceImage {
    fn drop(&mut self) {
        self.image_deallocate_channel_sender
            .send(self.image)
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

#[derive(Copy, Clone)]
struct ImageSetLayoutBase {
    sub_image_width: u32,
    sub_image_height: u32,
    max_image_size: u32,
}

impl ImageSetLayoutBase {
    fn new(sub_image_width: u32, sub_image_height: u32, max_image_size: u32) -> Result<Self> {
        if !sub_image_width.is_power_of_two() || !sub_image_height.is_power_of_two() {
            return Err(GLES2Error::ImageMustHavePowerOfTwoDimensions);
        }
        assert!(max_image_size.is_power_of_two());
        if sub_image_width > max_image_size || sub_image_height > max_image_size {
            Err(GLES2Error::ImageIsTooBig)
        } else {
            Ok(Self {
                sub_image_width: sub_image_width,
                sub_image_height: sub_image_height,
                max_image_size: max_image_size,
            })
        }
    }
    fn get_max_sub_image_count(&self) -> u32 {
        // glsl only guarantees that integers up to 2^10 are representable as mediump int
        const MAX_COUNT: u32 = 1 << 10;
        let max_sub_image_count_x = self.max_image_size / self.sub_image_width;
        let max_sub_image_count_y = self.max_image_size / self.sub_image_height;
        let retval = max_sub_image_count_x
            .checked_mul(max_sub_image_count_y)
            .and_then(|v| v.checked_mul(ShaderUniformLocations::SAMPLERS_LEN))
            .unwrap_or(MAX_COUNT);
        if retval > MAX_COUNT {
            MAX_COUNT
        } else {
            retval
        }
    }
}

#[derive(Copy, Clone)]
struct ImageSetLayout {
    base: ImageSetLayoutBase,
    sub_image_count_x: u32,
    sub_image_count_y: u32,
    sub_image_count: u32,
    image_count: u32,
}

struct FragmentShaderImageSetParameters {
    sampler_index_scale: f32,
    texture_coord_scale: [f32; 2],
    texture_index_scale: [f32; 2],
}

impl ImageSetLayout {
    fn new(base: ImageSetLayoutBase, sub_image_count: u32) -> Result<Self> {
        if sub_image_count > base.get_max_sub_image_count() {
            return Err(GLES2Error::ImageSetHasTooManyImages);
        }
        let mut sub_image_count_x = base.max_image_size / base.sub_image_width;
        let mut sub_image_count_y = base.max_image_size / base.sub_image_height;
        loop {
            if sub_image_count_x > 1
                && (sub_image_count_y <= 1
                    || sub_image_count_x * base.sub_image_width
                        >= sub_image_count_y * base.sub_image_height)
            {
                let new_sub_image_count_x = sub_image_count_x / 2;
                if new_sub_image_count_x
                    .checked_mul(sub_image_count_y)
                    .and_then(|v| v.checked_mul(ShaderUniformLocations::SAMPLERS_LEN))
                    .map(|v| v >= sub_image_count)
                    .unwrap_or(true)
                {
                    sub_image_count_x = new_sub_image_count_x;
                } else {
                    break;
                }
            } else if sub_image_count_y > 1 {
                let new_sub_image_count_y = sub_image_count_y / 2;
                if sub_image_count_x
                    .checked_mul(new_sub_image_count_y)
                    .and_then(|v| v.checked_mul(ShaderUniformLocations::SAMPLERS_LEN))
                    .map(|v| v >= sub_image_count)
                    .unwrap_or(true)
                {
                    sub_image_count_y = new_sub_image_count_y;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(Self {
            base: base,
            sub_image_count_x: sub_image_count_x,
            sub_image_count_y: sub_image_count_y,
            sub_image_count: sub_image_count,
            image_count: (sub_image_count + sub_image_count_x * sub_image_count_y - 1)
                / (sub_image_count_x * sub_image_count_y),
        })
    }
    fn get_fragment_shader_parameters(&self) -> FragmentShaderImageSetParameters {
        FragmentShaderImageSetParameters {
            sampler_index_scale: 1.0 / (self.sub_image_count_x * self.sub_image_count_y) as f32,
            texture_coord_scale: [
                1.0 / self.sub_image_count_x as f32,
                1.0 / self.sub_image_count_y as f32,
            ],
            texture_index_scale: [self.sub_image_count_x as f32, self.sub_image_count_y as f32],
        }
    }
}

pub struct GLES2StagingImageSet {
    device_image_set: GLES2DeviceImageSet,
    images: [Option<Image>; ShaderUniformLocations::SAMPLERS_LEN as usize],
}

impl GLES2StagingImageSet {
    fn make_images<T, F: FnMut(usize) -> T>(
        mut f: F,
    ) -> [T; ShaderUniformLocations::SAMPLERS_LEN as usize] {
        [f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7)]
    }
    fn new(layout: ImageSetLayout) -> Self {
        let images = Self::make_images(|index| {
            if index < layout.image_count as usize {
                Some(Image::new(
                    layout.base.sub_image_width * layout.sub_image_count_x,
                    layout.base.sub_image_height * layout.sub_image_count_y,
                    math::Vec4::new(0xFF, 0, 0xFF, 0xFF),
                ))
            } else {
                None
            }
        });
        let device_images = Self::make_images(|index| {
            if index < layout.image_count as usize {
                Some(None)
            } else {
                None
            }
        });
        Self {
            device_image_set: GLES2DeviceImageSet(Arc::new(GLES2DeviceImageSetState {
                images: Mutex::new(device_images),
                load_submitted_flag: None,
                layout: layout,
            })),
            images: images,
        }
    }
}

impl StagingImageSet for GLES2StagingImageSet {
    type DeviceImageSet = GLES2DeviceImageSet;
    fn get_device_image_set(&self) -> &GLES2DeviceImageSet {
        &self.device_image_set
    }
    fn write(&mut self, texture_id: TextureId, image: &image::Image) {
        let layout = &self.device_image_set.0.layout;
        assert!(texture_id != 0);
        let mut image_index = texture_id as u32 - 1;
        assert!(image_index < layout.sub_image_count);
        assert!(image.width() == layout.base.sub_image_width);
        assert!(image.height() == layout.base.sub_image_height);
        let x_sub_image_index = image_index % layout.sub_image_count_x;
        image_index /= layout.sub_image_count_x;
        let y_sub_image_index = image_index % layout.sub_image_count_y;
        image_index /= layout.sub_image_count_y;
        self.images[image_index as usize]
            .as_mut()
            .unwrap()
            .copy_area_from(
                x_sub_image_index * layout.base.sub_image_width,
                y_sub_image_index * layout.base.sub_image_height,
                image,
                0,
                0,
                layout.base.sub_image_width,
                layout.base.sub_image_height,
            );
    }
}

struct DeviceImageSetLockedState {
    images: [Option<Option<DeviceImage>>; ShaderUniformLocations::SAMPLERS_LEN as usize],
    load_submitted_flag: Option<Arc<atomic::AtomicBool>>,
}

struct DeviceImageSetState {
    locked_state: Mutex<DeviceImageSetLockedState>,
    layout: ImageSetLayout,
}

#[derive(Clone)]
pub struct GLES2DeviceImageSet(Arc<DeviceImageSetState>);

impl DeviceImageSet for GLES2DeviceImageSet {
    fn width(&self) -> u32 {
        self.0.layout.base.sub_image_width
    }
    fn height(&self) -> u32 {
        self.0.layout.base.sub_image_height
    }
    fn count(&self) -> u32 {
        self.0.layout.sub_image_count
    }
}

#[derive(Clone)]
pub struct GLES2DeviceReference {
    max_image_size: u32,
}

enum LoaderCommand {
    CopyVertexBufferToDevice(GLES2StagingVertexBuffer),
    CopyIndexBufferToDevice(GLES2StagingIndexBuffer),
    CopyImageSetToDevice(GLES2StagingImageSet),
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
    type StagingImageSet = GLES2StagingImageSet;
    type DeviceImageSet = GLES2DeviceImageSet;
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
    fn copy_image_set_to_device(&mut self, staging_image_set: GLES2StagingImageSet) -> Result<()> {
        let mut images = [None, None, None, None, None, None, None, None];
        for i in 0..(ShaderUniformLocations::SAMPLERS_LEN as usize) {
            images[i] = if staging_image_set.images[i].is_some() {
                Some(None)
            } else {
                None
            };
        }
        staging_image_set
            .device_image_set
            .0
            .locked_state
            .lock()
            .unwrap()
            .load_submitted_flag = Some(self.command_buffer.submitted_flag.clone());
        self.command_buffer
            .commands
            .push(LoaderCommand::CopyImageSetToDevice(staging_image_set));
        Ok(())
    }
}

enum RenderCommand {
    Draw {
        vertex_buffer: GLES2DeviceVertexBuffer,
        index_buffer: GLES2DeviceIndexBuffer,
        image_set: GLES2DeviceImageSet,
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
    image_set: Option<GLES2DeviceImageSet>,
    initial_transform: math::Mat4<f32>,
}

impl GLES2RenderCommandBufferBuilder {
    fn new() -> Self {
        Self {
            state: RenderCommandBufferState {
                commands: Vec::new(),
            },
            buffers: None,
            image_set: None,
            initial_transform: math::Mat4::identity(),
        }
    }
}

impl RenderCommandBufferBuilder for GLES2RenderCommandBufferBuilder {
    type Error = GLES2Error;
    type CommandBuffer = GLES2RenderCommandBuffer;
    type DeviceVertexBuffer = GLES2DeviceVertexBuffer;
    type DeviceIndexBuffer = GLES2DeviceIndexBuffer;
    type DeviceImageSet = GLES2DeviceImageSet;
    fn set_image_set(&mut self, image_set: Self::DeviceImageSet) {
        self.image_set = Some(image_set);
    }
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
        let image_set = self
            .image_set
            .clone()
            .expect("can't draw without image set bound");
        assert!(index_count as usize <= index_buffer.len());
        assert!(index_count as usize + first_index as usize <= index_buffer.len());
        assert!((vertex_offset as usize) < vertex_buffer.len() || index_count == 0);
        assert!(index_count % 3 == 0, "must be whole number of triangles");
        if index_count == 0 {
            return;
        }
        self.state.commands.push(RenderCommand::Draw {
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            image_set: image_set,
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
    type StagingImageSet = GLES2StagingImageSet;
    type DeviceImageSet = GLES2DeviceImageSet;
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
    fn get_max_image_width(&self) -> u32 {
        self.max_image_size
    }
    fn get_max_image_height(&self) -> u32 {
        self.max_image_size
    }
    fn get_max_image_count_in_image_set(&self, width: u32, height: u32) -> Result<u32> {
        Ok(ImageSetLayoutBase::new(width, height, self.max_image_size)?.get_max_sub_image_count())
    }
    fn create_staging_image_set(
        &self,
        width: u32,
        height: u32,
        count: u32,
    ) -> Result<GLES2StagingImageSet> {
        Ok(GLES2StagingImageSet::new(ImageSetLayout::new(
            ImageSetLayoutBase::new(width, height, self.max_image_size)?,
            count,
        )?))
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

#[derive(Clone, Copy)]
struct ShaderUniformLocations {
    initial_transform: api::GLint,
    final_transform: api::GLint,
    samplers: api::GLint,
    sampler_index_scale: api::GLint,
    texture_coord_scale: api::GLint,
    texture_index_scale: api::GLint,
}

impl ShaderUniformLocations {
    const SAMPLERS_LEN: u32 = 8;
}

pub struct GLES2Device {
    device_reference: GLES2DeviceReference,
    surface_state: SurfaceState,
    gl_context: GLContextWrapper,
    buffer_deallocate_channel_sender: mpsc::Sender<api::GLuint>,
    buffer_deallocate_channel_receiver: mpsc::Receiver<api::GLuint>,
    image_deallocate_channel_sender: mpsc::Sender<api::GLuint>,
    image_deallocate_channel_receiver: mpsc::Receiver<api::GLuint>,
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
    fn allocate_image(&mut self) -> DeviceImage {
        match self.image_deallocate_channel_receiver.try_recv() {
            Ok(image) => DeviceImage {
                image: image,
                image_deallocate_channel_sender: self.image_deallocate_channel_sender.clone(),
            },
            Err(mpsc::TryRecvError::Empty) => unsafe {
                let api = &self.gl_context.api;
                let mut image = 0;
                api.glGenTextures.unwrap()(1, &mut image);
                DeviceImage {
                    image: image,
                    image_deallocate_channel_sender: self.image_deallocate_channel_sender.clone(),
                }
            },
            _ => panic!(),
        }
    }
}

type Result<T> = result::Result<T, GLES2Error>;

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
    type StagingImageSet = GLES2StagingImageSet;
    type DeviceImageSet = GLES2DeviceImageSet;
    fn pause(self) -> GLES2PausedDevice {
        GLES2PausedDevice {
            surface_state: self.surface_state,
        }
    }
    fn resume(paused_device: GLES2PausedDevice) -> Result<Self> {
        let vertex_shader_source: &'static CStr = CStr::from_bytes_with_nul(
            concat!(
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/main.vert")),
                "\0"
            ).as_bytes(),
        ).unwrap();
        let fragment_shader_source: &'static CStr = CStr::from_bytes_with_nul(
            concat!(
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/main.frag")),
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
            let mut max_image_size;
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
                    buffer.resize(length as usize, 0);
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
                    buffer.resize(length as usize, 0);
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
                    (
                        initial_transform,
                        final_transform,
                        samplers,
                        sampler_index_scale,
                        texture_coord_scale,
                        texture_index_scale,
                    )
                );
                max_image_size = 0;
                api.glGetIntegerv.unwrap()(api::GL_MAX_TEXTURE_SIZE, &mut max_image_size);
                assert!(max_image_size > 0 && (max_image_size as u32).is_power_of_two());
            }
            let (buffer_deallocate_channel_sender, buffer_deallocate_channel_receiver) =
                mpsc::channel();
            let (image_deallocate_channel_sender, image_deallocate_channel_receiver) =
                mpsc::channel();
            Ok(GLES2Device {
                device_reference: GLES2DeviceReference {
                    max_image_size: max_image_size as u32,
                },
                surface_state: SurfaceState { window: window },
                gl_context: gl_context,
                buffer_deallocate_channel_sender: buffer_deallocate_channel_sender,
                buffer_deallocate_channel_receiver: buffer_deallocate_channel_receiver,
                image_deallocate_channel_sender: image_deallocate_channel_sender,
                image_deallocate_channel_receiver: image_deallocate_channel_receiver,
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
                    LoaderCommand::CopyImageSetToDevice(GLES2StagingImageSet {
                        device_image_set: GLES2DeviceImageSet(device_image_set_state),
                        images: staging_images,
                    }) => {
                        let DeviceImageSetState {
                            locked_state: locked_state,
                            layout: _,
                        } = &*device_image_set_state;
                        let mut locked_state = locked_state.lock().unwrap();
                        let DeviceImageSetLockedState {
                            images: device_images,
                            load_submitted_flag: _,
                        } = &mut *locked_state;
                        unsafe {
                            for i in 0..(ShaderUniformLocations::SAMPLERS_LEN as usize) {
                                match (&staging_images[i], &mut device_images[i]) {
                                    (Some(staging_image), Some(device_image)) => {
                                        let new_image = self.allocate_image();
                                        let api = &self.gl_context.api;
                                        api.glBindTexture.unwrap()(
                                            api::GL_TEXTURE_2D,
                                            new_image.image,
                                        );
                                        api.glTexParameteri.unwrap()(
                                            api::GL_TEXTURE_2D,
                                            api::GL_TEXTURE_MIN_FILTER,
                                            api::GL_NEAREST as api::GLint,
                                        );
                                        api.glTexParameteri.unwrap()(
                                            api::GL_TEXTURE_2D,
                                            api::GL_TEXTURE_MAG_FILTER,
                                            api::GL_NEAREST as api::GLint,
                                        );
                                        api.glTexParameteri.unwrap()(
                                            api::GL_TEXTURE_2D,
                                            api::GL_TEXTURE_WRAP_S,
                                            api::GL_REPEAT as api::GLint,
                                        );
                                        api.glTexParameteri.unwrap()(
                                            api::GL_TEXTURE_2D,
                                            api::GL_TEXTURE_WRAP_T,
                                            api::GL_REPEAT as api::GLint,
                                        );
                                        api.glTexImage2D.unwrap()(
                                            api::GL_TEXTURE_2D,
                                            0,
                                            api::GL_RGBA as api::GLint,
                                            staging_image.width() as api::GLsizei,
                                            staging_image.height() as api::GLsizei,
                                            0,
                                            api::GL_RGBA,
                                            api::GL_UNSIGNED_BYTE,
                                            (staging_image.get_pixels() as &[math::Vec4<u8>])
                                                .as_ptr()
                                                as *const c_void,
                                        );
                                        *device_image = Some(new_image);
                                    }
                                    (None, None) => {}
                                    _ => unreachable!(),
                                }
                            }
                        }
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
                                image_set: GLES2DeviceImageSet(image_set_state),
                                initial_transform,
                                index_count,
                                first_index,
                                vertex_offset,
                            } => {
                                let DeviceImageSetState {
                                    locked_state: image_set_locked_state,
                                    layout: image_set_layout,
                                } = &**image_set_state;
                                let image_set_locked_state = image_set_locked_state.lock();
                                let DeviceImageSetLockedState {
                                    images: image_set_images,
                                    load_submitted_flag: image_set_load_submitted_flag,
                                } = &*image_set_locked_state;
                                assert!(
                                    vertex_buffer_load_submitted_flag
                                        .load(atomic::Ordering::Acquire)
                                );
                                assert!(
                                    index_buffer_load_submitted_flag
                                        .load(atomic::Ordering::Acquire)
                                );
                                assert!(
                                    image_set_load_submitted_flag.load(atomic::Ordering::Acquire)
                                );
                                set_uniform_matrix(
                                    self.shader_uniform_locations.initial_transform,
                                    *initial_transform,
                                );
                                let mut textures = [0 as api::GLint;
                                    ShaderUniformLocations::SAMPLERS_LEN as usize];
                                let image_set_images = image_set_images.lock().unwrap();
                                for i in 0..ShaderUniformLocations::SAMPLERS_LEN {
                                    api.glActiveTexture.unwrap()(api::GL_TEXTURE0 + i);
                                    api.glBindTexture.unwrap()(
                                        api::GL_TEXTURE_2D,
                                        image_set_images[i as usize]
                                            .as_ref()
                                            .map(|image| image.as_ref().unwrap().image)
                                            .unwrap_or(0),
                                    );
                                    textures[i as usize] = i as api::GLint;
                                }
                                api.glUniform1iv.unwrap()(
                                    self.shader_uniform_locations.samplers,
                                    ShaderUniformLocations::SAMPLERS_LEN as api::GLsizei,
                                    &textures as *const _,
                                );
                                let FragmentShaderImageSetParameters {
                                    sampler_index_scale,
                                    texture_coord_scale,
                                    texture_index_scale,
                                } = image_set_layout.get_fragment_shader_parameters();
                                api.glUniform1f.unwrap()(
                                    self.shader_uniform_locations.sampler_index_scale,
                                    sampler_index_scale,
                                );
                                api.glUniform2f.unwrap()(
                                    self.shader_uniform_locations.texture_coord_scale,
                                    texture_coord_scale[0],
                                    texture_coord_scale[1],
                                );
                                api.glUniform2f.unwrap()(
                                    self.shader_uniform_locations.texture_index_scale,
                                    texture_index_scale[0],
                                    texture_index_scale[1],
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
                                        texture_id,
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
