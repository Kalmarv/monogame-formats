//! The compiled effect (MGFX) an XNB of MonoGame's `EffectReader` holds: version 12, written by the Byting Games fork
//! of MonoGame 3.8.1 (upstream MonoGame releases write 10 and 11). Shaders are the GLSL MojoShader wrote from the
//! effect's Shader Model 3 programs (profile OpenGL); a DirectX 11 profile byte is read the same way, but only OpenGL
//! files were checked. Written against the 18 effects of STUNTBOOST (Byting Games); other versions are refused by
//! number.
//!
//! Layout (little-endian; strings are .NET's, a 7-bit encoded length then UTF-8):
//! - header: `MGFX`, the version (12), the profile (0 OpenGL, 1 DirectX 11), the effect key (i32), then a byte that is
//!   1 in every file checked;
//! - constant buffers: an i32 count, each its name, a byte 0, its size in bytes (u16), a byte 0, an i32 count of
//!   parameters and for each the parameter's index (i32) and its offset in the buffer (u16);
//! - shaders: an i32 count, each the stage (a byte, 0 vertex 1 pixel), the source file name and entry point (strings,
//!   empty in every file checked), a byte 0, the code (an i32 length and the bytes), the samplers (a byte count; each the
//!   kind, texture slot and sampler slot, whether it has a state, that state's 20 bytes, then its name and the index of
//!   the parameter that holds its texture), the constant buffers it reads (a byte count, each a byte index), its vertex
//!   attributes (a byte count; each its name, usage, usage index, location (i16) and a byte 1);
//! - parameters: an i32 count, each its class and type (bytes), name and semantic, an i32 annotation count (0), rows and
//!   columns (bytes), its elements and its structure members (parameters again), and for a plain bool, int or float its
//!   `rows * columns` default values (floats; MojoShader stores every scalar as one);
//! - techniques: an i32 count, each its name, an i32 annotation count (0) and its passes (an i32 count; each its name,
//!   an i32 annotation count (0), six i32 shader indices (vertex, pixel, then four more, -1 when the pass has none) and
//!   three bytes saying whether it has a blend, a depth-stencil and a rasterizer state);
//! - the tail, `MGFX` again, at the end of the effect's bytes.
//!
//! A pass with a state, another version, a byte that is not what every file checked has, an
//! annotation or a string parameter is refused, naming it: none of those is in any file checked, and their layout is not
//! known.
#![deny(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation
)]

use crate::xnb::{Content, Error};

type Result<T> = std::result::Result<T, Error>;

/// The version read.
pub const VERSION: u8 = 12;
const SIGNATURE: &[u8; 4] = b"MGFX";

/// What the effect's shaders are written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    /// GLSL text (MojoShader's output for OpenGL).
    OpenGl,
    /// DXBC.
    DirectX11,
}

/// The pipeline stage a shader runs in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    /// A vertex shader.
    Vertex,
    /// A pixel (fragment) shader.
    Pixel,
}

/// A constant buffer: the effect's parameters at offsets in a block of `size` bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstantBuffer {
    /// The buffer's name in the shader's code (`ps_uniforms_vec4`).
    pub name: String,
    /// The block's size in bytes.
    pub size: u16,
    /// (parameter index in [`Effect::parameters`], byte offset).
    pub parameters: Vec<(u32, u16)>,
}

/// What a sampler samples (MonoGame's `SamplerType`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SamplerKind {
    /// A 2D texture.
    Sampler2D,
    /// A cube map.
    SamplerCube,
    /// A volume (3D) texture.
    SamplerVolume,
    /// A 1D texture.
    Sampler1D,
}

/// A sampler state an effect states (`SamplerState`): the address modes (MonoGame's `TextureAddressMode`: 0 wrap, 1
/// clamp, 2 mirror, 3 border), the border colour, the filter (`TextureFilter`: 0 linear, 1 point, 2 anisotropic, ...),
/// the anisotropy, the largest mip level and the mip bias.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SamplerState {
    /// The address modes, three bytes of `TextureAddressMode` as stored.
    pub address: [u8; 3],
    /// The border colour, four bytes as stored.
    pub border: [u8; 4],
    /// `TextureFilter`, as stored.
    pub filter: u8,
    /// The anisotropy limit.
    pub max_anisotropy: i32,
    /// The largest mip level used.
    pub max_mip_level: i32,
    /// The mip level bias.
    pub mip_bias: f32,
}

/// A texture sampler a shader reads.
#[derive(Clone, Debug, PartialEq)]
pub struct Sampler {
    /// What the sampler samples.
    pub kind: SamplerKind,
    /// The texture slot, as stored.
    pub texture_slot: u8,
    /// The sampler slot, as stored.
    pub sampler_slot: u8,
    /// The sampler state the effect states, if it states one.
    pub state: Option<SamplerState>,
    /// The sampler's name in the shader's code (`ps_s0`).
    pub name: String,
    /// The index in [`Effect::parameters`] of the parameter that holds the texture.
    pub parameter: u8,
}

/// A vertex attribute of a vertex shader: MonoGame's `VertexElementUsage` (0 position, 1 color, 2 texture
/// coordinate, 3 normal, 4 binormal, 5 tangent, 6 blend indices, 7 blend weight, 8 depth, 9 fog, 10 point size, 11
/// sample, 12 tessellate factor) and index, and the name and location the code gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    /// The attribute's name in the shader's code (`vs_v0`).
    pub name: String,
    /// `VertexElementUsage`, as stored.
    pub usage: u8,
    /// The usage index (the second texture coordinate is usage 2, index 1).
    pub index: u8,
    /// The attribute's location in the code.
    pub location: i16,
}

/// One compiled shader.
#[derive(Clone, Debug, PartialEq)]
pub struct Shader<'a> {
    /// The stage it runs in.
    pub stage: Stage,
    /// The source file name as stored (empty in the files this crate was written against).
    pub source_file: String,
    /// The entry point as stored (empty in the files this crate was written against).
    pub entry_point: String,
    /// The GLSL (or DXBC) as stored.
    pub code: &'a [u8],
    /// The samplers it reads.
    pub samplers: Vec<Sampler>,
    /// The indices in [`Effect::constant_buffers`] of the buffers it reads.
    pub constant_buffers: Vec<u8>,
    /// The vertex attributes it takes (vertex shaders).
    pub attributes: Vec<Attribute>,
}

/// An effect parameter. `class` is MonoGame's `EffectParameterClass` (0 scalar, 1 vector, 2 matrix, 3 object, 4
/// struct) and `ty` its `EffectParameterType` (1 bool, 2 int, 3 float, 7 texture 2D, 9 texture cube, ...).
#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    /// `EffectParameterClass`, as stored.
    pub class: u8,
    /// `EffectParameterType`, as stored.
    pub ty: u8,
    /// The parameter's name.
    pub name: String,
    /// The semantic as stored, empty when it has none.
    pub semantic: String,
    /// Rows, as stored.
    pub rows: u8,
    /// Columns, as stored.
    pub columns: u8,
    /// The elements of an array parameter.
    pub elements: Vec<Parameter>,
    /// The members of a struct parameter.
    pub members: Vec<Parameter>,
    /// The defaults of a plain bool, int or float, `rows * columns` of them.
    pub defaults: Option<Vec<f32>>,
}

/// One pass of a technique: the shaders it runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pass {
    /// The pass's name.
    pub name: String,
    /// Index in [`Effect::shaders`] of the vertex shader, if the pass has one.
    pub vertex_shader: Option<usize>,
    /// Index in [`Effect::shaders`] of the pixel shader, if the pass has one.
    pub pixel_shader: Option<usize>,
}

/// A technique: a named sequence of passes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Technique {
    /// The technique's name.
    pub name: String,
    /// Its passes, in order.
    pub passes: Vec<Pass>,
}

/// A compiled effect, read by [`Effect::parse`]. Every index it holds into another table is checked to be in range.
#[derive(Clone, Debug, PartialEq)]
pub struct Effect<'a> {
    /// What the shaders are written in.
    pub profile: Profile,
    /// The effect key, as stored.
    pub key: i32,
    /// The constant buffers.
    pub constant_buffers: Vec<ConstantBuffer>,
    /// The shaders, in the order passes index them.
    pub shaders: Vec<Shader<'a>>,
    /// The parameters.
    pub parameters: Vec<Parameter>,
    /// The techniques.
    pub techniques: Vec<Technique>,
}

fn bad(offset: usize, detail: impl Into<String>) -> Error {
    Error::Effect {
        offset: offset as u128,
        detail: detail.into(),
    }
}

/// An error of the byte reader as one of the effect's.
fn effect(e: Error) -> Error {
    match e {
        Error::Format { offset, detail } => Error::Effect { offset, detail },
        other => other,
    }
}

/// The effect's bytes read in order, through the one reader of .NET's encodings ([`Content`]).
struct Reader<'a> {
    content: Content<'a>,
}

impl<'a> Reader<'a> {
    fn position(&self) -> usize {
        self.content.position()
    }

    fn remaining(&self) -> usize {
        self.content.remaining()
    }

    fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        self.content.bytes(n).map_err(effect)
    }

    fn u8(&mut self) -> Result<u8> {
        self.content.u8().map_err(effect)
    }

    fn i16(&mut self) -> Result<i16> {
        self.content.i16().map_err(effect)
    }

    fn u16(&mut self) -> Result<u16> {
        self.content.u16().map_err(effect)
    }

    fn i32(&mut self) -> Result<i32> {
        self.content.i32().map_err(effect)
    }

    fn f32(&mut self) -> Result<f32> {
        self.content.f32().map_err(effect)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.content.array().map_err(effect)
    }

    /// A count: an i32 that is not negative and cannot promise more items than there are bytes left (each item is at
    /// least a byte).
    fn count(&mut self, what: &str) -> Result<usize> {
        let at = self.position();
        let n = self.i32()?;
        match usize::try_from(n) {
            Ok(n) if n <= self.remaining() => Ok(n),
            _ => Err(bad(at, format!("{what}: a count of {n}"))),
        }
    }

    fn string(&mut self) -> Result<String> {
        Ok(self.content.string().map_err(effect)?.to_owned())
    }

    /// A byte that must be `expected` in every file checked.
    fn constant(&mut self, expected: u8, what: &str) -> Result<()> {
        let at = self.position();
        let b = self.u8()?;
        if b == expected {
            Ok(())
        } else {
            Err(bad(
                at,
                format!("{what}: {b}, where every file checked has {expected} (the layout is not known)"),
            ))
        }
    }

    fn no_annotations(&mut self, what: &str) -> Result<()> {
        let at = self.position();
        match self.i32()? {
            0 => Ok(()),
            n => Err(bad(at, format!("{what}: {n} annotations are not read"))),
        }
    }

    fn parameters(&mut self, depth: u32) -> Result<Vec<Parameter>> {
        if depth > 8 {
            return Err(bad(self.position(), "parameters nested over 8 deep"));
        }
        let count = self.count("parameters")?;
        let mut out = Vec::new();
        for _ in 0..count {
            let class = self.u8()?;
            let ty_at = self.position();
            let ty = self.u8()?;
            let name = self.string()?;
            let semantic = self.string()?;
            self.no_annotations(&name)?;
            let rows = self.u8()?;
            let columns = self.u8()?;
            let elements = self.parameters(depth.saturating_add(1))?;
            let members = self.parameters(depth.saturating_add(1))?;
            let defaults = if elements.is_empty() && members.is_empty() {
                match ty {
                    // bool, int and float: MojoShader stores each as a float
                    1..=3 => {
                        let n = usize::from(rows).saturating_mul(usize::from(columns));
                        let mut v = Vec::new();
                        for _ in 0..n {
                            v.push(self.f32()?);
                        }
                        Some(v)
                    }
                    4 => return Err(bad(ty_at, format!("{name}: a string parameter is not read"))),
                    _ => None,
                }
            } else {
                None
            };
            out.push(Parameter {
                class,
                ty,
                name,
                semantic,
                rows,
                columns,
                elements,
                members,
                defaults,
            });
        }
        Ok(out)
    }

    fn sampler(&mut self) -> Result<Sampler> {
        let at = self.position();
        let kind = match self.u8()? {
            0 => SamplerKind::Sampler2D,
            1 => SamplerKind::SamplerCube,
            2 => SamplerKind::SamplerVolume,
            3 => SamplerKind::Sampler1D,
            other => return Err(bad(at, format!("sampler kind {other}"))),
        };
        let texture_slot = self.u8()?;
        let sampler_slot = self.u8()?;
        let state_at = self.position();
        let state = match self.u8()? {
            0 => None,
            1 => Some(SamplerState {
                address: self.array()?,
                border: self.array()?,
                filter: self.u8()?,
                max_anisotropy: self.i32()?,
                max_mip_level: self.i32()?,
                mip_bias: self.f32()?,
            }),
            other => return Err(bad(state_at, format!("a sampler state flag of {other}"))),
        };
        Ok(Sampler {
            kind,
            texture_slot,
            sampler_slot,
            state,
            name: self.string()?,
            parameter: self.u8()?,
        })
    }

    /// One shader. `buffers` is how many constant buffers the effect has: the shader's buffer indices are checked
    /// against it here. Each sampler's parameter index is pushed to `sampler_parameters` (offset, index, sampler name)
    /// to be checked once the parameter table has been read.
    fn shader(
        &mut self,
        buffers: usize,
        sampler_parameters: &mut Vec<(usize, u8, String)>,
    ) -> Result<Shader<'a>> {
        let at = self.position();
        let stage = match self.u8()? {
            0 => Stage::Vertex,
            1 => Stage::Pixel,
            other => return Err(bad(at, format!("shader stage {other}"))),
        };
        let source_file = self.string()?;
        let entry_point = self.string()?;
        self.constant(0, "the byte after a shader's entry point")?;
        let at = self.position();
        let length = self.i32()?;
        let length = usize::try_from(length).map_err(|_| bad(at, format!("shader length {length}")))?;
        let code = self.bytes(length)?;
        let mut samplers = Vec::new();
        for _ in 0..self.u8()? {
            let sampler = self.sampler()?;
            // the sampler's last byte is its parameter index
            sampler_parameters.push((
                self.position().saturating_sub(1),
                sampler.parameter,
                sampler.name.clone(),
            ));
            samplers.push(sampler);
        }
        let mut constant_buffers = Vec::new();
        for _ in 0..self.u8()? {
            let at = self.position();
            let b = self.u8()?;
            if usize::from(b) >= buffers {
                return Err(bad(
                    at,
                    format!(
                        "a shader reads constant buffer {b} of {buffers} (0 to {})",
                        buffers.saturating_sub(1)
                    ),
                ));
            }
            constant_buffers.push(b);
        }
        let mut attributes = Vec::new();
        for _ in 0..self.u8()? {
            let attribute = Attribute {
                name: self.string()?,
                usage: self.u8()?,
                index: self.u8()?,
                location: self.i16()?,
            };
            self.constant(1, "the byte after a vertex attribute")?;
            attributes.push(attribute);
        }
        self.constant(0, "the byte after a shader's attributes")?;
        Ok(Shader {
            stage,
            source_file,
            entry_point,
            code,
            samplers,
            constant_buffers,
            attributes,
        })
    }

    fn pass(&mut self, shaders: usize) -> Result<Pass> {
        let name = self.string()?;
        self.no_annotations(&name)?;
        let index = |lift: &mut Self, what: &str| -> Result<Option<usize>> {
            let at = lift.position();
            match lift.i32()? {
                -1 => Ok(None),
                n => match usize::try_from(n) {
                    Ok(n) if n < shaders => Ok(Some(n)),
                    _ => Err(bad(at, format!("{what} shader {n} of {shaders}"))),
                },
            }
        };
        let vertex_shader = index(self, "vertex")?;
        let pixel_shader = index(self, "pixel")?;
        for stage in ["hull", "domain", "geometry", "compute"] {
            let at = self.position();
            if index(self, stage)?.is_some() {
                return Err(bad(at, format!("a pass with a {stage} shader is not read")));
            }
        }
        for state in ["blend", "depth-stencil", "rasterizer"] {
            let at = self.position();
            if self.u8()? != 0 {
                return Err(bad(at, format!("a pass with a {state} state is not read")));
            }
        }
        Ok(Pass {
            name,
            vertex_shader,
            pixel_shader,
        })
    }
}

impl<'a> Effect<'a> {
    /// Parse the effect `code` as `EffectReader` stores it (the bytes of an XNB's effect, see
    /// [`crate::xnb::Xnb::effect`]). The whole of `code` is read.
    pub fn parse(code: &'a [u8]) -> Result<Effect<'a>> {
        let mut r = Reader {
            content: Content::of_bytes(code),
        };
        if r.bytes(4)? != SIGNATURE {
            return Err(bad(0, "not an MGFX effect"));
        }
        let version = r.u8()?;
        if version != VERSION {
            return Err(bad(
                4,
                format!("MGFX version {version}: only {VERSION} (BytingGames' MonoGame 3.8.1) is read"),
            ));
        }
        let profile = match r.u8()? {
            0 => Profile::OpenGl,
            1 => Profile::DirectX11,
            other => return Err(bad(5, format!("MGFX profile {other}"))),
        };
        let key = r.i32()?;
        r.constant(1, "the byte after the effect key")?;
        let mut constant_buffers = Vec::new();
        // (offset, index, buffer name) of each parameter index a buffer holds
        let mut buffer_parameters = Vec::new();
        for _ in 0..r.count("constant buffers")? {
            let name = r.string()?;
            r.constant(0, "the byte before a constant buffer's size")?;
            let size = r.u16()?;
            r.constant(0, "the byte after a constant buffer's size")?;
            let mut parameters = Vec::new();
            for _ in 0..r.count("constant buffer parameters")? {
                let at = r.position();
                let index = r.i32()?;
                let index = u32::try_from(index).map_err(|_| bad(at, format!("parameter index {index}")))?;
                buffer_parameters.push((at, index, name.clone()));
                parameters.push((index, r.u16()?));
            }
            constant_buffers.push(ConstantBuffer {
                name,
                size,
                parameters,
            });
        }
        let mut shaders = Vec::new();
        let mut sampler_parameters = Vec::new();
        for _ in 0..r.count("shaders")? {
            shaders.push(r.shader(constant_buffers.len(), &mut sampler_parameters)?);
        }
        let parameters = r.parameters(0)?;
        let mut techniques = Vec::new();
        for _ in 0..r.count("techniques")? {
            let name = r.string()?;
            r.no_annotations(&name)?;
            let mut passes = Vec::new();
            for _ in 0..r.count("passes")? {
                passes.push(r.pass(shaders.len())?);
            }
            techniques.push(Technique { name, passes });
        }
        if r.bytes(4)? != SIGNATURE {
            return Err(bad(
                r.position().saturating_sub(4),
                "the effect does not end with MGFX",
            ));
        }
        if r.remaining() != 0 {
            return Err(bad(
                r.position(),
                format!("{} bytes after the effect", r.remaining()),
            ));
        }
        // every parameter index a buffer or a sampler holds is in range
        for (at, index, owner) in buffer_parameters {
            if usize::try_from(index).is_ok_and(|i| i < parameters.len()) {
                continue;
            }
            return Err(bad(
                at,
                format!(
                    "constant buffer {owner} holds parameter {index} of {} (0 to {})",
                    parameters.len(),
                    parameters.len().saturating_sub(1)
                ),
            ));
        }
        for (at, index, owner) in sampler_parameters {
            if usize::from(index) >= parameters.len() {
                return Err(bad(
                    at,
                    format!(
                        "sampler {owner} names parameter {index} of {} (0 to {})",
                        parameters.len(),
                        parameters.len().saturating_sub(1)
                    ),
                ));
            }
        }
        Ok(Effect {
            profile,
            key,
            constant_buffers,
            shaders,
            parameters,
            techniques,
        })
    }
}

#[cfg(test)]
#[allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::unwrap_used,
    reason = "fixtures, not untrusted bytes"
)]
mod tests {
    use super::*;

    fn string(s: &str, out: &mut Vec<u8>) {
        assert!(s.len() < 128);
        out.push(s.len() as u8);
        out.extend_from_slice(s.as_bytes());
    }

    /// A small effect: one constant buffer holding one parameter, a vertex shader with two attributes and a pixel
    /// shader with a sampler, one technique with one pass.
    fn effect(version: u8, profile: u8, state: u8) -> Vec<u8> {
        let mut b = b"MGFX".to_vec();
        b.extend_from_slice(&[version, profile]);
        b.extend_from_slice(&0x1234_5678i32.to_le_bytes());
        b.push(1);
        // constant buffers
        b.extend_from_slice(&1i32.to_le_bytes());
        string("ps_uniforms_vec4", &mut b);
        b.push(0);
        b.extend_from_slice(&16i16.to_le_bytes());
        b.push(0);
        b.extend_from_slice(&1i32.to_le_bytes());
        b.extend_from_slice(&0i32.to_le_bytes());
        b.extend_from_slice(&0u16.to_le_bytes());
        // shaders: the pixel shader (stage 1) then the vertex shader (stage 0)
        b.extend_from_slice(&2i32.to_le_bytes());
        for (stage, code) in [(1u8, "void main() {}\n"), (0u8, "void main() { }\n")] {
            b.push(stage);
            string("", &mut b);
            string("", &mut b);
            b.push(0);
            b.extend_from_slice(&(code.len() as i32).to_le_bytes());
            b.extend_from_slice(code.as_bytes());
            if stage == 1 {
                // a sampler: 2D, slots 3 and 3, a state, its name and its parameter
                b.push(1);
                b.extend_from_slice(&[0, 3, 3, 1]);
                b.extend_from_slice(&[1, 1, 0, 255, 255, 255, 255, 1]);
                b.extend_from_slice(&4i32.to_le_bytes());
                b.extend_from_slice(&0i32.to_le_bytes());
                b.extend_from_slice(&(-16.0f32).to_le_bytes());
                string("ps_s0", &mut b);
                b.push(1);
                b.extend_from_slice(&[1, 0]);
                b.push(0);
            } else {
                b.push(0);
                b.push(0);
                b.push(2);
                for (name, usage, index) in [("vs_v0", 0u8, 0u8), ("vs_v1", 2, 1)] {
                    string(name, &mut b);
                    b.extend_from_slice(&[usage, index]);
                    b.extend_from_slice(&(index as i16).to_le_bytes());
                    b.push(1);
                }
            }
            b.push(0);
        }
        // parameters: a float4 and a texture
        b.extend_from_slice(&2i32.to_le_bytes());
        b.extend_from_slice(&[1, 3]);
        string("Tint", &mut b);
        string("", &mut b);
        b.extend_from_slice(&0i32.to_le_bytes());
        b.extend_from_slice(&[1, 4]);
        b.extend_from_slice(&0i32.to_le_bytes());
        b.extend_from_slice(&0i32.to_le_bytes());
        for v in [0.25f32, 0.5, 0.75, 1.0] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        b.extend_from_slice(&[3, 7]);
        string("Albedo", &mut b);
        string("", &mut b);
        b.extend_from_slice(&0i32.to_le_bytes());
        b.extend_from_slice(&[1, 1]);
        b.extend_from_slice(&0i32.to_le_bytes());
        b.extend_from_slice(&0i32.to_le_bytes());
        // techniques
        b.extend_from_slice(&1i32.to_le_bytes());
        string("Render", &mut b);
        b.extend_from_slice(&0i32.to_le_bytes());
        b.extend_from_slice(&1i32.to_le_bytes());
        string("Pass", &mut b);
        b.extend_from_slice(&0i32.to_le_bytes());
        for index in [1i32, 0, -1, -1, -1, -1] {
            b.extend_from_slice(&index.to_le_bytes());
        }
        b.extend_from_slice(&[state, 0, 0]);
        b.extend_from_slice(b"MGFX");
        b
    }

    #[test]
    fn an_effect_reads_to_its_shaders_parameters_and_passes() {
        let bytes = effect(VERSION, 0, 0);
        let e = Effect::parse(&bytes).unwrap();
        assert_eq!(e.profile, Profile::OpenGl);
        assert_eq!(e.key, 0x1234_5678);
        assert_eq!(e.constant_buffers.len(), 1);
        assert_eq!(e.constant_buffers[0].name, "ps_uniforms_vec4");
        assert_eq!(e.constant_buffers[0].size, 16);
        assert_eq!(e.shaders.len(), 2);
        let (ps, vs) = (&e.shaders[0], &e.shaders[1]);
        assert_eq!((ps.stage, vs.stage), (Stage::Pixel, Stage::Vertex));
        assert_eq!(ps.code, b"void main() {}\n");
        assert_eq!(ps.samplers.len(), 1);
        assert_eq!(ps.samplers[0].name, "ps_s0");
        assert_eq!(ps.samplers[0].kind, SamplerKind::Sampler2D);
        assert_eq!(ps.samplers[0].state.unwrap().mip_bias, -16.0);
        assert_eq!(ps.constant_buffers, [0]);
        assert_eq!(vs.attributes.len(), 2);
        assert_eq!((vs.attributes[1].usage, vs.attributes[1].index), (2, 1));
        assert_eq!(
            e.parameters[0].defaults.as_deref(),
            Some(&[0.25, 0.5, 0.75, 1.0][..])
        );
        assert_eq!(e.parameters[1].defaults, None);
        let pass = &e.techniques[0].passes[0];
        assert_eq!((pass.vertex_shader, pass.pixel_shader), (Some(1), Some(0)));
    }

    #[test]
    fn what_is_not_read_is_refused_by_name() {
        let wrong_version = Effect::parse(&effect(11, 0, 0)).unwrap_err().to_string();
        assert!(wrong_version.contains("MGFX version 11"), "{wrong_version}");
        let bytes = effect(VERSION, 1, 0);
        let direct_x = Effect::parse(&bytes).unwrap();
        assert_eq!(direct_x.profile, Profile::DirectX11);
        let state = Effect::parse(&effect(VERSION, 0, 1)).unwrap_err().to_string();
        assert!(state.contains("blend state"), "{state}");
        let mut cut = effect(VERSION, 0, 0);
        cut.truncate(cut.len() - 9);
        assert!(Effect::parse(&cut).is_err());
        let mut extra = effect(VERSION, 0, 0);
        extra.push(0);
        let extra = Effect::parse(&extra).unwrap_err().to_string();
        assert!(extra.contains("after the effect"), "{extra}");
    }

    #[test]
    fn an_effect_error_is_the_effects_and_a_constant_buffer_names_only_parameters_it_has() {
        let message = Effect::parse(b"XXXX").unwrap_err().to_string();
        assert!(
            message.contains("MGFX effect at byte 0 of its bytes"),
            "{message}"
        );
        // the constant buffer's one parameter, index 0, made 9 (the effect has two)
        let mut bytes = effect(VERSION, 0, 0);
        let at = bytes.windows(16).position(|w| w == b"ps_uniforms_vec4").unwrap();
        // the name, a byte, the size (i16), a byte, the count (i32), then the index (i32)
        let index = at + 16 + 1 + 2 + 1 + 4;
        assert_eq!(bytes[index..index + 4], 0i32.to_le_bytes());
        bytes[index] = 9;
        let message = Effect::parse(&bytes).unwrap_err().to_string();
        assert!(
            message.contains("constant buffer ps_uniforms_vec4 holds parameter 9 of 2"),
            "{message}"
        );
    }

    #[test]
    fn a_seven_bit_length_over_32_bits_is_refused_wherever_it_is_read() {
        // the constant buffer's name, whose one-byte length (16) is five bytes that carry bits above the 32nd
        let mut bytes = effect(VERSION, 0, 0);
        let at = bytes.windows(16).position(|w| w == b"ps_uniforms_vec4").unwrap() - 1;
        assert_eq!(bytes[at], 16);
        bytes.splice(at..=at, [0x90, 0x80, 0x80, 0x80, 0x7F]);
        let message = Effect::parse(&bytes).unwrap_err().to_string();
        assert!(message.contains("over 32 bits"), "{message}");
    }

    /// The offset an effect error names, and its detail.
    fn error_of(bytes: &[u8]) -> (u128, String) {
        match Effect::parse(bytes).unwrap_err() {
            Error::Effect { offset, detail } => (offset, detail),
            other => panic!("not an effect error: {other}"),
        }
    }

    fn find(bytes: &[u8], needle: &[u8]) -> usize {
        bytes.windows(needle.len()).position(|w| w == needle).unwrap()
    }

    #[test]
    fn a_constant_buffer_size_is_the_two_bytes_stored_unsigned() {
        let mut bytes = effect(VERSION, 0, 0);
        let size = find(&bytes, b"ps_uniforms_vec4") + 16 + 1;
        assert_eq!(bytes[size..size + 2], 16i16.to_le_bytes());
        bytes[size..size + 2].copy_from_slice(&40_000u16.to_le_bytes());
        let e = Effect::parse(&bytes).unwrap();
        assert_eq!(e.constant_buffers[0].size, 40_000);
    }

    #[test]
    fn an_index_out_of_range_names_the_bytes_that_hold_it() {
        let good = effect(VERSION, 0, 0);
        // a constant buffer's parameter index (i32), after its name, a byte, the size, a byte and the count
        let at = find(&good, b"ps_uniforms_vec4") + 16 + 1 + 2 + 1 + 4;
        let mut bytes = good.clone();
        bytes[at] = 9;
        let (offset, detail) = error_of(&bytes);
        assert_eq!(offset, at as u128, "{detail}");
        assert!(detail.contains("holds parameter 9 of 2"), "{detail}");
        // a sampler's parameter byte, after its name
        let at = find(&good, b"ps_s0") + 5;
        assert_eq!(good[at], 1);
        let mut bytes = good.clone();
        bytes[at] = 7;
        let (offset, detail) = error_of(&bytes);
        assert_eq!(offset, at as u128, "{detail}");
        assert!(detail.contains("ps_s0 names parameter 7 of 2"), "{detail}");
        // a shader's constant buffer index, after the sampler's parameter and the count
        let at = find(&good, b"ps_s0") + 5 + 1 + 1;
        assert_eq!(good[at], 0);
        let mut bytes = good;
        bytes[at] = 3;
        let (offset, detail) = error_of(&bytes);
        assert_eq!(offset, at as u128, "{detail}");
        assert!(detail.contains("reads constant buffer 3 of 1"), "{detail}");
    }

    #[test]
    fn a_string_parameter_is_refused_at_its_type_byte() {
        let mut bytes = effect(VERSION, 0, 0);
        // "Albedo": its length byte, before that the type byte
        let at = find(&bytes, b"Albedo") - 1 - 1;
        assert_eq!(bytes[at], 7);
        bytes[at] = 4;
        let (offset, detail) = error_of(&bytes);
        assert_eq!(offset, at as u128, "{detail}");
        assert!(detail.contains("Albedo: a string parameter"), "{detail}");
    }

    #[test]
    fn counts_and_lengths_the_bytes_cannot_hold_are_errors() {
        // every i32 of the fixture made i32::MAX in turn: an error or a read, never a panic
        let good = effect(VERSION, 0, 0);
        for at in 0..good.len() - 3 {
            let mut bytes = good.clone();
            bytes[at..at + 4].copy_from_slice(&i32::MAX.to_le_bytes());
            let _ = Effect::parse(&bytes);
        }
        // a shader of 2 GiB in a few bytes
        let mut bytes = good;
        let code = find(&bytes, b"void main() {}") - 4;
        bytes[code..code + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        let (offset, _) = error_of(&bytes);
        assert!(offset >= code as u128);
    }

    #[test]
    fn every_truncation_is_refused() {
        let bytes = effect(VERSION, 0, 0);
        for n in 0..bytes.len() {
            assert!(Effect::parse(&bytes[..n]).is_err(), "prefix {n}");
        }
    }
}
