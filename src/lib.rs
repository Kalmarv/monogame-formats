//! Read MonoGame content files: the XNB container and the compiled MGFX effects inside it.
//!
//! MonoGame (and XNA 4.0, which FNA also reads) ships a game's content as `.xnb` files. This crate reads them from
//! bytes you give it, without writing, decompressing or running any game code:
//!
//! - [`xnb`]: the XNB container (version 5, uncompressed): its header, its type-reader table as stored, and the
//!   primary object's bytes, with reads for the values XNA's built-in readers write (7-bit encoded lengths, strings,
//!   byte arrays, raw `Color` textures). A game's own readers are the caller's to name: the container hands their
//!   bytes over.
//! - [`mgfx`]: the compiled effect an XNB of MonoGame's `EffectReader` holds (MGFX version 12, OpenGL or DirectX 11
//!   profile): constant buffers, shaders with their samplers and attributes, parameters, techniques and passes.
//!
//! Every read is checked: untrusted bytes give an [`xnb::Error`] naming the offset and what was expected, never a
//! panic. What the crate does not read is refused by name rather than guessed at (compressed files, other versions,
//! shared resources, effect passes with render state).
//!
//! # Example
//!
//! ```
//! use monogame_formats::xnb::Xnb;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // A one-string XNB, built by hand: header, one type reader, the object.
//! let reader = "Microsoft.Xna.Framework.Content.StringReader";
//! let mut file = vec![b'X', b'N', b'B', b'd', 5, 0, 0, 0, 0, 0, 1, reader.len() as u8];
//! file.extend_from_slice(reader.as_bytes());
//! file.extend_from_slice(&[0, 0, 0, 0]); // the reader's version
//! file.extend_from_slice(&[0, 1, 5]); // no shared resources, type id 1, a 5-byte string
//! file.extend_from_slice(b"hello");
//! let size = file.len() as u32;
//! file[6..10].copy_from_slice(&size.to_le_bytes()); // the header holds the file's size
//!
//! let xnb = Xnb::parse(&file)?;
//! assert_eq!(xnb.readers[0].type_name(), reader);
//! assert_eq!(xnb.string()?, "hello");
//! # Ok(())
//! # }
//! ```
//!
//! For an effect, [`xnb::Xnb::effect`] returns the compiled effect's bytes and [`mgfx::Effect::parse`] reads them.
#![deny(clippy::disallowed_methods, clippy::disallowed_types)]
#![warn(missing_docs)]

pub mod mgfx;
pub mod xnb;
