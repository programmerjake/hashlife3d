use super::super::sdl;
use super::*;
use std::error;
use std::ffi::CStr;
use std::fmt;
use std::mem;
use std::os::raw::*;
use std::ptr::*;
use std::result;

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
        }
    }
}

impl error::Error for GLES2Error {}

pub struct GLES2Semaphore {}

impl Semaphore for GLES2Semaphore {}

pub struct GLES2Fence {}

impl Fence for GLES2Fence {}

pub struct GLES2Queue {}

impl Queue for GLES2Queue {}

#[derive(Clone)]
pub struct GLES2DeviceReference {}

impl DeviceReference for GLES2DeviceReference {
    type Semaphore = GLES2Semaphore;
    type Fence = GLES2Fence;
    type Error = GLES2Error;
    fn create_fence(&self, initial_state: FenceState) -> Result<GLES2Fence> {
        unimplemented!()
    }
}

struct SurfaceState {
    window: sdl::window::Window,
}

pub struct GLES2PausedDevice {
    surface_state: SurfaceState,
}

pub struct GLES2Device {
    device_reference: GLES2DeviceReference,
    surface_state: SurfaceState,
    render_queue: GLES2Queue,
}

pub type Result<T> = result::Result<T, GLES2Error>;

impl PausedDevice for GLES2PausedDevice {
    type Device = GLES2Device;
    fn get_window(&self) -> &sdl::window::Window {
        &self.surface_state.window
    }
}

impl Device for GLES2Device {
    type Semaphore = GLES2Semaphore;
    type Fence = GLES2Fence;
    type Error = GLES2Error;
    type Reference = GLES2DeviceReference;
    type Queue = GLES2Queue;
    type PausedDevice = GLES2PausedDevice;
    fn pause(self) -> GLES2PausedDevice {
        GLES2PausedDevice {
            surface_state: self.surface_state,
        }
    }
    fn resume(paused_device: GLES2PausedDevice) -> Result<Self> {
        let SurfaceState { window } = paused_device.surface_state;
        unsafe {
            set_gl_attributes()?;
        }
        Ok(GLES2Device {
            device_reference: GLES2DeviceReference {},
            surface_state: SurfaceState { window: window },
            render_queue: GLES2Queue {},
        })
    }
    fn get_device_ref(&self) -> &GLES2DeviceReference {
        &self.device_reference
    }
    fn get_window(&self) -> &sdl::window::Window {
        &self.surface_state.window
    }
    fn get_queue(&self) -> &GLES2Queue {
        &self.render_queue
    }
    fn wait_for_fences_with_timeout(
        &self,
        _fences: &[&GLES2Fence],
        _wait_for_all: bool,
        _timeout: Duration,
    ) -> Result<WaitResult> {
        Ok(WaitResult::Success)
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
