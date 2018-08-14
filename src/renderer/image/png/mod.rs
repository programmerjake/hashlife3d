#[cfg(test)]
mod tests;
use super::{Image, ImageLoader};
use inflate::DeflateDecoder;
use renderer::math;
use std::ascii;
use std::borrow::BorrowMut;
use std::error;
use std::fmt;
use std::hash::Hasher;
use std::io::{self, prelude::*, Error, ErrorKind, Result};
use std::iter::Peekable;
use std::mem;

pub struct PngImageLoader {
    max_chunk_length: usize,
    max_width: u32,
    max_height: u32,
    max_pixel_count: usize,
}

impl Default for PngImageLoader {
    fn default() -> Self {
        Self {
            max_chunk_length: 0x100000,
            max_width: 0x10000,
            max_height: 0x10000,
            max_pixel_count: (1 << 30) / 4, // 1GiB of RAM for 4-byte pixels
        }
    }
}

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

#[derive(Debug)]
struct CrcHasher(u32);

impl CrcHasher {
    const LOOKUP_TABLE: [u32; 256] = [
        0x00000000, 0x77073096, 0xEE0E612C, 0x990951BA, 0x076DC419, 0x706AF48F, 0xE963A535,
        0x9E6495A3, 0x0EDB8832, 0x79DCB8A4, 0xE0D5E91E, 0x97D2D988, 0x09B64C2B, 0x7EB17CBD,
        0xE7B82D07, 0x90BF1D91, 0x1DB71064, 0x6AB020F2, 0xF3B97148, 0x84BE41DE, 0x1ADAD47D,
        0x6DDDE4EB, 0xF4D4B551, 0x83D385C7, 0x136C9856, 0x646BA8C0, 0xFD62F97A, 0x8A65C9EC,
        0x14015C4F, 0x63066CD9, 0xFA0F3D63, 0x8D080DF5, 0x3B6E20C8, 0x4C69105E, 0xD56041E4,
        0xA2677172, 0x3C03E4D1, 0x4B04D447, 0xD20D85FD, 0xA50AB56B, 0x35B5A8FA, 0x42B2986C,
        0xDBBBC9D6, 0xACBCF940, 0x32D86CE3, 0x45DF5C75, 0xDCD60DCF, 0xABD13D59, 0x26D930AC,
        0x51DE003A, 0xC8D75180, 0xBFD06116, 0x21B4F4B5, 0x56B3C423, 0xCFBA9599, 0xB8BDA50F,
        0x2802B89E, 0x5F058808, 0xC60CD9B2, 0xB10BE924, 0x2F6F7C87, 0x58684C11, 0xC1611DAB,
        0xB6662D3D, 0x76DC4190, 0x01DB7106, 0x98D220BC, 0xEFD5102A, 0x71B18589, 0x06B6B51F,
        0x9FBFE4A5, 0xE8B8D433, 0x7807C9A2, 0x0F00F934, 0x9609A88E, 0xE10E9818, 0x7F6A0DBB,
        0x086D3D2D, 0x91646C97, 0xE6635C01, 0x6B6B51F4, 0x1C6C6162, 0x856530D8, 0xF262004E,
        0x6C0695ED, 0x1B01A57B, 0x8208F4C1, 0xF50FC457, 0x65B0D9C6, 0x12B7E950, 0x8BBEB8EA,
        0xFCB9887C, 0x62DD1DDF, 0x15DA2D49, 0x8CD37CF3, 0xFBD44C65, 0x4DB26158, 0x3AB551CE,
        0xA3BC0074, 0xD4BB30E2, 0x4ADFA541, 0x3DD895D7, 0xA4D1C46D, 0xD3D6F4FB, 0x4369E96A,
        0x346ED9FC, 0xAD678846, 0xDA60B8D0, 0x44042D73, 0x33031DE5, 0xAA0A4C5F, 0xDD0D7CC9,
        0x5005713C, 0x270241AA, 0xBE0B1010, 0xC90C2086, 0x5768B525, 0x206F85B3, 0xB966D409,
        0xCE61E49F, 0x5EDEF90E, 0x29D9C998, 0xB0D09822, 0xC7D7A8B4, 0x59B33D17, 0x2EB40D81,
        0xB7BD5C3B, 0xC0BA6CAD, 0xEDB88320, 0x9ABFB3B6, 0x03B6E20C, 0x74B1D29A, 0xEAD54739,
        0x9DD277AF, 0x04DB2615, 0x73DC1683, 0xE3630B12, 0x94643B84, 0x0D6D6A3E, 0x7A6A5AA8,
        0xE40ECF0B, 0x9309FF9D, 0x0A00AE27, 0x7D079EB1, 0xF00F9344, 0x8708A3D2, 0x1E01F268,
        0x6906C2FE, 0xF762575D, 0x806567CB, 0x196C3671, 0x6E6B06E7, 0xFED41B76, 0x89D32BE0,
        0x10DA7A5A, 0x67DD4ACC, 0xF9B9DF6F, 0x8EBEEFF9, 0x17B7BE43, 0x60B08ED5, 0xD6D6A3E8,
        0xA1D1937E, 0x38D8C2C4, 0x4FDFF252, 0xD1BB67F1, 0xA6BC5767, 0x3FB506DD, 0x48B2364B,
        0xD80D2BDA, 0xAF0A1B4C, 0x36034AF6, 0x41047A60, 0xDF60EFC3, 0xA867DF55, 0x316E8EEF,
        0x4669BE79, 0xCB61B38C, 0xBC66831A, 0x256FD2A0, 0x5268E236, 0xCC0C7795, 0xBB0B4703,
        0x220216B9, 0x5505262F, 0xC5BA3BBE, 0xB2BD0B28, 0x2BB45A92, 0x5CB36A04, 0xC2D7FFA7,
        0xB5D0CF31, 0x2CD99E8B, 0x5BDEAE1D, 0x9B64C2B0, 0xEC63F226, 0x756AA39C, 0x026D930A,
        0x9C0906A9, 0xEB0E363F, 0x72076785, 0x05005713, 0x95BF4A82, 0xE2B87A14, 0x7BB12BAE,
        0x0CB61B38, 0x92D28E9B, 0xE5D5BE0D, 0x7CDCEFB7, 0x0BDBDF21, 0x86D3D2D4, 0xF1D4E242,
        0x68DDB3F8, 0x1FDA836E, 0x81BE16CD, 0xF6B9265B, 0x6FB077E1, 0x18B74777, 0x88085AE6,
        0xFF0F6A70, 0x66063BCA, 0x11010B5C, 0x8F659EFF, 0xF862AE69, 0x616BFFD3, 0x166CCF45,
        0xA00AE278, 0xD70DD2EE, 0x4E048354, 0x3903B3C2, 0xA7672661, 0xD06016F7, 0x4969474D,
        0x3E6E77DB, 0xAED16A4A, 0xD9D65ADC, 0x40DF0B66, 0x37D83BF0, 0xA9BCAE53, 0xDEBB9EC5,
        0x47B2CF7F, 0x30B5FFE9, 0xBDBDF21C, 0xCABAC28A, 0x53B39330, 0x24B4A3A6, 0xBAD03605,
        0xCDD70693, 0x54DE5729, 0x23D967BF, 0xB3667A2E, 0xC4614AB8, 0x5D681B02, 0x2A6F2B94,
        0xB40BBE37, 0xC30C8EA1, 0x5A05DF1B, 0x2D02EF8D,
    ];
}

impl Hasher for CrcHasher {
    fn finish(&self) -> u64 {
        (!self.0) as u64
    }
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = Self::LOOKUP_TABLE[((self.0 ^ byte as u32) & 0xFF) as usize] ^ (self.0 >> 8);
        }
    }
}

impl Default for CrcHasher {
    fn default() -> CrcHasher {
        CrcHasher(!0)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct ChunkName([u8; ChunkName::BYTE_COUNT]);

impl fmt::Debug for ChunkName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_representation: String = self
            .0
            .iter()
            .flat_map(|v| ascii::escape_default(*v).map(|v| v as char))
            .collect();
        f.debug_struct("ChunkName")
            .field("bytes", &self.0)
            .field("bytes", &string_representation)
            .finish()
    }
}

impl ChunkName {
    const BYTE_COUNT: usize = 4;
    fn is_ancillary(&self) -> bool {
        (self.0[0] & 0x20) != 0
    }
    fn is_private(&self) -> bool {
        (self.0[1] & 0x20) != 0
    }
    #[allow(dead_code)]
    fn is_safe_to_copy(&self) -> bool {
        (self.0[3] & 0x20) != 0
    }
    fn is_critical(&self) -> bool {
        !self.is_ancillary()
    }
    #[allow(dead_code)]
    fn is_public(&self) -> bool {
        !self.is_private()
    }
    const IHDR: ChunkName = ChunkName(*b"IHDR");
    const PLTE: ChunkName = ChunkName(*b"PLTE");
    const IDAT: ChunkName = ChunkName(*b"IDAT");
    const IEND: ChunkName = ChunkName(*b"IEND");
    #[allow(non_upper_case_globals)]
    const tRNS: ChunkName = ChunkName(*b"tRNS");
}

fn read_all_or_none<R: Read, T: BorrowMut<[u8]>>(
    reader: &mut R,
    mut bytes: T,
) -> Result<Option<T>> {
    let is_none;
    {
        let bytes_slice = bytes.borrow_mut();
        let read_count = reader.read(bytes_slice)?;
        is_none = read_count == 0;
        if !is_none && read_count < bytes_slice.len() {
            reader.read_exact(&mut bytes_slice[read_count..])?;
        }
    }
    if is_none {
        Ok(None)
    } else {
        Ok(Some(bytes))
    }
}

fn read_all<R: Read, T: BorrowMut<[u8]>>(reader: &mut R, bytes: T) -> Result<T> {
    read_all_or_none(reader, bytes)?.ok_or_else(|| ErrorKind::UnexpectedEof.into())
}

fn read_u8<R: Read>(reader: &mut R) -> Result<u8> {
    read_all(reader, [0]).map(|v| v[0])
}

fn read_u32_or_none_unchecked_range<R: Read>(reader: &mut R) -> Result<Option<u32>> {
    Ok(read_all_or_none(reader, [0; 4])?.map(|v| u32::from_be(unsafe { mem::transmute(v) })))
}

fn read_u32_unchecked_range<R: Read>(reader: &mut R) -> Result<u32> {
    read_u32_or_none_unchecked_range(reader)?.ok_or_else(|| ErrorKind::UnexpectedEof.into())
}

fn read_i32_unchecked_range<R: Read>(reader: &mut R) -> Result<i32> {
    Ok(i32::from_be(unsafe {
        mem::transmute(read_all(reader, [0; 4])?)
    }))
}

fn read_chunk_name<R: Read>(reader: &mut R) -> Result<ChunkName> {
    Ok(ChunkName(read_all(reader, [0; 4])?))
}

fn read_u32_or_none<R: Read>(reader: &mut R) -> Result<Option<u32>> {
    if let Some(retval) = read_u32_or_none_unchecked_range(reader)? {
        if retval >= (1 << 31) {
            return Err(Error::new(ErrorKind::InvalidData, "value out of range"));
        }
        Ok(Some(retval))
    } else {
        Ok(None)
    }
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32> {
    read_u32_or_none(reader)?.ok_or_else(|| ErrorKind::UnexpectedEof.into())
}

fn read_i32<R: Read>(reader: &mut R) -> Result<i32> {
    let retval = read_i32_unchecked_range(reader)?;
    if retval == (-1 << 31) {
        return Err(Error::new(ErrorKind::InvalidData, "value out of range"));
    }
    Ok(retval)
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum ColorType {
    Grayscale = 0x0,
    RGB = 0x2,
    RGBPalette = 0x3,
    GrayscaleAlpha = 0x4,
    RGBA = 0x6,
}

impl ColorType {
    fn from(v: u8) -> Option<ColorType> {
        let enumerants = [
            ColorType::Grayscale,
            ColorType::RGB,
            ColorType::RGBPalette,
            ColorType::GrayscaleAlpha,
            ColorType::RGBA,
        ];
        for &enumerant in &enumerants {
            if v == enumerant as u8 {
                return Some(enumerant);
            }
        }
        None
    }
    fn is_allowed_bit_depth(&self, bit_depth: u8) -> bool {
        match (self, bit_depth) {
            (ColorType::Grayscale, 1)
            | (ColorType::Grayscale, 2)
            | (ColorType::Grayscale, 4)
            | (ColorType::Grayscale, 8)
            | (ColorType::Grayscale, 16)
            | (ColorType::RGB, 8)
            | (ColorType::RGB, 16)
            | (ColorType::RGBPalette, 1)
            | (ColorType::RGBPalette, 2)
            | (ColorType::RGBPalette, 4)
            | (ColorType::RGBPalette, 8)
            | (ColorType::GrayscaleAlpha, 8)
            | (ColorType::GrayscaleAlpha, 16)
            | (ColorType::RGBA, 8)
            | (ColorType::RGBA, 16) => true,
            _ => false,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum ScanlineFilterAlgorithm {
    None = 0x0,
    Sub = 0x1,
    Up = 0x2,
    Average = 0x3,
    Paeth = 0x4,
}

impl ScanlineFilterAlgorithm {
    fn from(v: u8) -> Option<ScanlineFilterAlgorithm> {
        let enumerants = [
            ScanlineFilterAlgorithm::None,
            ScanlineFilterAlgorithm::Sub,
            ScanlineFilterAlgorithm::Up,
            ScanlineFilterAlgorithm::Average,
            ScanlineFilterAlgorithm::Paeth,
        ];
        for &enumerant in &enumerants {
            if v == enumerant as u8 {
                return Some(enumerant);
            }
        }
        None
    }
    fn filter_scanline(
        self,
        prev_scanline: &[u8],
        scanline: &mut [u8],
        pixel_size_in_bytes: usize,
    ) {
        assert_eq!(prev_scanline.len(), scanline.len());
        const DEBUG_PRINT: bool = false;
        if DEBUG_PRINT {
            println!(
                "filter_scanline: self = {:?}, pixel_size_in_bytes = {}",
                self, pixel_size_in_bytes,
            );
            println!(
                "prev_scanline:\n{:?}\nscanline:\n{:?}",
                prev_scanline, scanline,
            );
        }
        match self {
            ScanlineFilterAlgorithm::None => {}
            ScanlineFilterAlgorithm::Sub => for i in 0..scanline.len() {
                let left = if i < pixel_size_in_bytes {
                    0
                } else {
                    scanline[i - pixel_size_in_bytes]
                };
                scanline[i] = left.wrapping_add(scanline[i]);
            },
            ScanlineFilterAlgorithm::Up => for i in 0..scanline.len() {
                let up = prev_scanline[i];
                scanline[i] = up.wrapping_add(scanline[i]);
            },
            ScanlineFilterAlgorithm::Average => for i in 0..scanline.len() {
                let left = if i < pixel_size_in_bytes {
                    0
                } else {
                    scanline[i - pixel_size_in_bytes]
                };
                let up = prev_scanline[i];
                scanline[i] = (((up as u32 + left as u32) / 2) as u8).wrapping_add(scanline[i]);
            },
            ScanlineFilterAlgorithm::Paeth => for i in 0..scanline.len() {
                let left = if i < pixel_size_in_bytes {
                    0
                } else {
                    scanline[i - pixel_size_in_bytes]
                };
                let up_left = if i < pixel_size_in_bytes {
                    0
                } else {
                    prev_scanline[i - pixel_size_in_bytes]
                };
                let up = prev_scanline[i];
                fn paeth_predictor(a: i32, b: i32, c: i32) -> i32 {
                    let p = a + b - c;
                    let pa = (p - a).abs();
                    let pb = (p - b).abs();
                    let pc = (p - c).abs();
                    if pa <= pb && pa <= pc {
                        a
                    } else if pb <= pc {
                        b
                    } else {
                        c
                    }
                }
                scanline[i] = (paeth_predictor(left as i32, up as i32, up_left as i32) as u8)
                    .wrapping_add(scanline[i]);
            },
        }
        if DEBUG_PRINT {
            println!("scanline:\n{:?}", scanline,);
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum CompressionMethod {
    Deflate = 0x0,
}

impl CompressionMethod {
    fn from(v: u8) -> Option<CompressionMethod> {
        if v == CompressionMethod::Deflate as u8 {
            Some(CompressionMethod::Deflate)
        } else {
            None
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum FilterMethod {
    Adaptive = 0x0,
}

impl FilterMethod {
    fn from(v: u8) -> Option<FilterMethod> {
        if v == FilterMethod::Adaptive as u8 {
            Some(FilterMethod::Adaptive)
        } else {
            None
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum InterlaceMethod {
    NoInterlace = 0x0,
    Adam7 = 0x1,
}

impl InterlaceMethod {
    fn from(v: u8) -> Option<InterlaceMethod> {
        let enumerants = [InterlaceMethod::NoInterlace, InterlaceMethod::Adam7];
        for &enumerant in &enumerants {
            if v == enumerant as u8 {
                return Some(enumerant);
            }
        }
        None
    }
}

struct IHDRChunk {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: ColorType,
    compression_method: CompressionMethod,
    filter_method: FilterMethod,
    interlace_method: InterlaceMethod,
}

impl IHDRChunk {
    const WIDTH_FIELD_SIZE: usize = 4;
    const HEIGHT_FIELD_SIZE: usize = 4;
    const BIT_DEPTH_FIELD_SIZE: usize = 1;
    const COLOR_TYPE_FIELD_SIZE: usize = 1;
    const COMPRESSION_METHOD_FIELD_SIZE: usize = 1;
    const FILTER_METHOD_FIELD_SIZE: usize = 1;
    const INTERLACE_METHOD_FIELD_SIZE: usize = 1;
    const SIZE: usize = Self::WIDTH_FIELD_SIZE
        + Self::HEIGHT_FIELD_SIZE
        + Self::BIT_DEPTH_FIELD_SIZE
        + Self::COLOR_TYPE_FIELD_SIZE
        + Self::COMPRESSION_METHOD_FIELD_SIZE
        + Self::FILTER_METHOD_FIELD_SIZE
        + Self::INTERLACE_METHOD_FIELD_SIZE;
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let width = read_u32(reader)?;
        if width == 0 {
            return Err(Error::new(ErrorKind::InvalidData, "value out of range"));
        }
        let height = read_u32(reader)?;
        if height == 0 {
            return Err(Error::new(ErrorKind::InvalidData, "value out of range"));
        }
        let bit_depth = read_u8(reader)?;
        let color_type = ColorType::from(read_u8(reader)?)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "value out of range"))?;
        let compression_method = CompressionMethod::from(read_u8(reader)?)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "value out of range"))?;
        let filter_method = FilterMethod::from(read_u8(reader)?)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "value out of range"))?;
        let interlace_method = InterlaceMethod::from(read_u8(reader)?)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "value out of range"))?;
        if !color_type.is_allowed_bit_depth(bit_depth) {
            return Err(Error::new(ErrorKind::InvalidData, "value out of range"));
        }
        Ok(Self {
            width: width,
            height: height,
            bit_depth: bit_depth,
            color_type: color_type,
            compression_method: compression_method,
            filter_method: filter_method,
            interlace_method: interlace_method,
        })
    }
}

#[derive(Debug)]
struct Chunk {
    name: ChunkName,
    data: Vec<u8>,
}

impl Chunk {
    fn require(self, expected_name: ChunkName) -> Result<Self> {
        if self.name == expected_name {
            Ok(self)
        } else {
            #[derive(Debug)]
            struct RequiredChunkNameError {
                expected_name: ChunkName,
                found_name: ChunkName,
            }
            impl fmt::Display for RequiredChunkNameError {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(
                        f,
                        "missing chunk: {:?}: found {:?}",
                        self.expected_name, self.found_name
                    )
                }
            }
            impl error::Error for RequiredChunkNameError {}
            Err(Error::new(
                ErrorKind::InvalidData,
                RequiredChunkNameError {
                    expected_name: expected_name,
                    found_name: self.name,
                },
            ))
        }
    }
    fn require_length(self, expected_length: usize) -> Result<Self> {
        if self.data.len() == expected_length {
            Ok(self)
        } else {
            #[derive(Debug)]
            struct RequiredChunkLengthError {
                chunk_name: ChunkName,
                expected_length: usize,
                found_length: usize,
            }
            impl fmt::Display for RequiredChunkLengthError {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(
                        f,
                        "chunk: {:?} must have length {}: found {}",
                        self.chunk_name, self.expected_length, self.found_length
                    )
                }
            }
            impl error::Error for RequiredChunkLengthError {}
            Err(Error::new(
                ErrorKind::InvalidData,
                RequiredChunkLengthError {
                    chunk_name: self.name,
                    expected_length: expected_length,
                    found_length: self.data.len(),
                },
            ))
        }
    }
}

struct ChunkIterator<R: Read> {
    reader: R,
    max_chunk_length: usize,
}

impl<R: Read> ChunkIterator<R> {
    fn next_helper(&mut self) -> Result<Option<Chunk>> {
        if let Some(chunk_length) = read_u32_or_none(&mut self.reader)? {
            if chunk_length as usize > self.max_chunk_length {
                return Err(Error::new(ErrorKind::InvalidData, "chunk too big"));
            }
            let mut chunk = Chunk {
                name: read_chunk_name(&mut self.reader)?,
                data: Vec::new(),
            };
            chunk.data.resize(chunk_length as usize, 0);
            self.reader.read_exact(&mut chunk.data)?;
            let chunk_crc = read_u32_unchecked_range(&mut self.reader)?;
            let mut crc_hasher = CrcHasher::default();
            crc_hasher.write(&chunk.name.0);
            crc_hasher.write(&chunk.data);
            let computed_chunk_crc = crc_hasher.finish() as u32;
            if chunk_crc != computed_chunk_crc {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "chunk CRC doesn't match",
                ));
            }
            Ok(Some(chunk))
        } else {
            Ok(None)
        }
    }
}

impl<R: Read> Iterator for ChunkIterator<R> {
    type Item = Result<Chunk>;
    fn next(&mut self) -> Option<Result<Chunk>> {
        match self.next_helper() {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => None,
            Err(v) => Some(Err(v)),
        }
    }
}

struct CompressedByteReader<'a, R: 'a + Read> {
    chunk_iterator: &'a mut Peekable<ChunkIterator<R>>,
    chunk: Option<io::Cursor<Vec<u8>>>,
}

impl<'a, R: 'a + Read> Read for CompressedByteReader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        loop {
            if let Some(chunk) = self.chunk.as_mut() {
                match chunk.read(buf) {
                    Ok(0) => {}
                    result => return result,
                }
            }
            self.chunk = None;
            match self.chunk_iterator.peek() {
                Some(Ok(Chunk {
                    name: ChunkName::IDAT,
                    ..
                }))
                | Some(Err(_)) => {}
                _ => break,
            }
            self.chunk = Some(io::Cursor::new(self.chunk_iterator.next().unwrap()?.data));
        }
        Ok(0)
    }
}

struct ChunkingReader<R: Read> {
    underlying_reader: R,
    chunk: Option<io::Cursor<Vec<u8>>>,
    hit_end: bool,
}

impl<R: Read> ChunkingReader<R> {
    fn new(underlying_reader: R) -> Self {
        Self {
            underlying_reader: underlying_reader,
            chunk: Some(io::Cursor::new(Vec::new())),
            hit_end: false,
        }
    }
}

impl<R: Read> Read for ChunkingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        const CHUNK_SIZE: usize = 0x1000;
        loop {
            match self.chunk.as_mut().unwrap().read(buf) {
                Ok(0) => (),
                result => return result,
            }
            if self.hit_end {
                return Ok(0);
            }
            let mut chunk = self.chunk.take().unwrap().into_inner();
            chunk.resize(CHUNK_SIZE, 0);
            let mut position = 0;
            loop {
                if position >= chunk.len() {
                    chunk.resize(position, 0);
                    self.chunk = Some(io::Cursor::new(chunk));
                    break;
                }
                match self.underlying_reader.read(&mut chunk[position..]) {
                    Ok(current_count) if current_count != 0 => {
                        assert!(current_count <= chunk[position..].len());
                        position += current_count;
                    }
                    result => {
                        chunk.resize(position, 0);
                        self.chunk = Some(io::Cursor::new(chunk));
                        result?;
                        self.hit_end = true;
                        break;
                    }
                }
            }
        }
    }
}

impl ImageLoader for PngImageLoader {
    fn name(&self) -> &'static str {
        "png"
    }
    fn matches_signature<R: Read>(&self, reader: &mut R) -> Result<bool> {
        let mut signature = [0; 8];
        reader.read_exact(&mut signature)?;
        Ok(PNG_SIGNATURE == signature)
    }
    fn load<R: Read>(&self, reader: &mut R) -> Result<Image> {
        let mut signature = [0; 8];
        reader.read_exact(&mut signature)?;
        if PNG_SIGNATURE != signature {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "PNG signature didn't match",
            ));
        }
        let mut chunk_iterator = ChunkIterator {
            reader: reader,
            max_chunk_length: self.max_chunk_length,
        }.peekable();
        let ihdr_chunk = IHDRChunk::read(&mut io::Cursor::new(
            chunk_iterator
                .next()
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing IHDR chunk"))??
                .require(ChunkName::IHDR)?
                .require_length(IHDRChunk::SIZE)?
                .data,
        ))?;
        if ihdr_chunk.width > self.max_width || ihdr_chunk.height > self.max_height {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "image is too big: {}x{}",
                    ihdr_chunk.width, ihdr_chunk.height
                ),
            ));
        }
        if ihdr_chunk.width as u64 * ihdr_chunk.height as u64 > self.max_pixel_count as u64 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "image takes too much memory: {}x{}: {}Mpx",
                    ihdr_chunk.width,
                    ihdr_chunk.height,
                    (ihdr_chunk.width as f32 * ihdr_chunk.height as f32) / 1e6
                ),
            ));
        }
        fn handle_ignored_chunk(chunk: Chunk) -> Result<()> {
            if chunk.name == ChunkName::IHDR {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("IHDR chunk is not allowed here"),
                ))
            } else if chunk.name == ChunkName::PLTE {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("PLTE chunk is not allowed here"),
                ))
            } else if chunk.name == ChunkName::IDAT {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("IDAT chunk is not allowed here"),
                ))
            } else if chunk.name == ChunkName::IEND {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("IEND chunk is not allowed here"),
                ))
            } else if chunk.name == ChunkName::tRNS {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("tRNS chunk is not allowed here"),
                ))
            } else if chunk.name.is_critical() {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unknown critical chunk: {:?}", chunk.name),
                ))
            } else {
                Ok(())
            }
        }
        let mut palette = None;
        let mut trns_chunk = None;
        loop {
            if let Some(Ok(Chunk {
                name: ChunkName::IDAT,
                ..
            })) = chunk_iterator.peek()
            {
                break;
            }
            let mut chunk = chunk_iterator
                .next()
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing IDAT chunk"))??;
            match chunk.name {
                ChunkName::PLTE if palette.is_none() && trns_chunk.is_none() => {
                    if chunk.data.len() == 0 {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "PLTE must have non-zero length",
                        ));
                    }
                    if chunk.data.len() % 3 != 0 {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "PLTE must have length that is a multiple of 3",
                        ));
                    }
                    let max_palette_entry_count = match ihdr_chunk.color_type {
                        ColorType::RGBPalette => 1 << ihdr_chunk.bit_depth,
                        ColorType::RGB | ColorType::RGBA => 0x100,
                        _ => {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "PLTE not allowed for grayscale images",
                            ))
                        }
                    };
                    if chunk.data.len() > 3 * max_palette_entry_count {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "PLTE has too many entries",
                        ));
                    }
                    palette = Some(chunk.data);
                }
                ChunkName::tRNS if trns_chunk.is_none() => {
                    if chunk.data.len() == 0 {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "tRNS must have non-zero length",
                        ));
                    }
                    match ihdr_chunk.color_type {
                        ColorType::RGBPalette => {
                            let palette = palette.as_ref().ok_or_else(|| {
                                Error::new(
                                    ErrorKind::InvalidData,
                                    "tRNS must not precede PLTE chunk for indexed images",
                                )
                            })?;
                            if chunk.data.len() * 3 > palette.len() {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "tRNS must have fewer entries than the palette",
                                ));
                            }
                            chunk.data.resize(palette.len() / 3, 0xFF);
                        }
                        ColorType::Grayscale => {
                            if chunk.data.len() != 2 {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "tRNS must be 2 bytes for grayscale images",
                                ));
                            }
                            let value = ((chunk.data[0] as u32) << 8) | chunk.data[1] as u32;
                            if (value >> ihdr_chunk.bit_depth) != 0 {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "tRNS value is out of range",
                                ));
                            }
                        }
                        ColorType::RGB => {
                            if chunk.data.len() != 6 {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "tRNS must be 6 bytes for truecolor images",
                                ));
                            }
                            let r = ((chunk.data[0] as u32) << 8) | chunk.data[1] as u32;
                            let g = ((chunk.data[2] as u32) << 8) | chunk.data[3] as u32;
                            let b = ((chunk.data[4] as u32) << 8) | chunk.data[5] as u32;
                            if (r >> ihdr_chunk.bit_depth) != 0
                                || (g >> ihdr_chunk.bit_depth) != 0
                                || (b >> ihdr_chunk.bit_depth) != 0
                            {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "tRNS value is out of range",
                                ));
                            }
                        }
                        ColorType::GrayscaleAlpha | ColorType::RGBA => {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "tRNS chunk is not allowed for images with alpha channels",
                            ))
                        }
                    }
                    trns_chunk = Some(chunk.data);
                }
                _ => handle_ignored_chunk(chunk)?,
            }
        }
        if palette.is_none() && ihdr_chunk.color_type == ColorType::RGBPalette {
            return Err(Error::new(ErrorKind::InvalidData, "PLTE chunk not found"));
        }
        fn decode_image<R: Read>(
            width: u32,
            height: u32,
            bit_depth: u8,
            color_type: ColorType,
            palette: &Option<Vec<u8>>,
            trns_chunk: &Option<Vec<u8>>,
            reader: &mut R,
        ) -> Result<Option<Image>> {
            if width == 0 || height == 0 {
                return Ok(None);
            }
            let bits_per_pixel = match color_type {
                ColorType::Grayscale => bit_depth,
                ColorType::RGB => bit_depth * 3,
                ColorType::RGBPalette => bit_depth,
                ColorType::GrayscaleAlpha => bit_depth * 2,
                ColorType::RGBA => bit_depth * 4,
            };
            let rounded_up_bytes_per_pixel = (bits_per_pixel + 7) / 8;
            let image_bytes_per_scan_line = ((width as usize)
                .checked_mul(bits_per_pixel as usize)
                .unwrap()
                + 7)
                / 8;
            let mut last_scanline = Vec::new();
            last_scanline.resize(image_bytes_per_scan_line, 0); // last_scanline needs to start as all zeros
            let mut current_scanline = last_scanline.clone();
            let mut retval = Image::new(width, height, Default::default());
            for y in 0..height {
                let filter_algorithm =
                    ScanlineFilterAlgorithm::from(read_u8(reader)?).ok_or_else(|| {
                        Error::new(ErrorKind::InvalidData, "invalid scanline filter algorithm")
                    })?;
                reader.read_exact(&mut current_scanline)?;
                filter_algorithm.filter_scanline(
                    &last_scanline,
                    &mut current_scanline,
                    rounded_up_bytes_per_pixel as usize,
                );
                fn u16_to_u8(value: u16) -> u8 {
                    ((value as u32 * 0xFF + 0xFFFF / 2) / 0xFFFF) as u8
                }
                match (bit_depth, color_type, trns_chunk) {
                    (1, ColorType::Grayscale, None) => {
                        for x in 0..width {
                            let value = (current_scanline[(x / 8) as usize] >> (7 - x % 8)) & 0x1;
                            *retval.get_mut(x, y) =
                                math::Vec4::new(value * 0xFF, value * 0xFF, value * 0xFF, 0xFF);
                        }
                    }
                    (1, ColorType::Grayscale, Some(trns_chunk)) => {
                        let transparent_value = trns_chunk[1];
                        for x in 0..width {
                            let value = (current_scanline[(x / 8) as usize] >> (7 - x % 8)) & 0x1;
                            *retval.get_mut(x, y) = math::Vec4::new(
                                value * 0xFF,
                                value * 0xFF,
                                value * 0xFF,
                                if value == transparent_value { 0 } else { 0xFF },
                            );
                        }
                    }
                    (2, ColorType::Grayscale, None) => {
                        for x in 0..width {
                            let value =
                                (current_scanline[(x / 4) as usize] >> (3 - x % 4) * 2) & 0x3;
                            *retval.get_mut(x, y) =
                                math::Vec4::new(value * 0x55, value * 0x55, value * 0x55, 0xFF);
                        }
                    }
                    (2, ColorType::Grayscale, Some(trns_chunk)) => {
                        let transparent_value = trns_chunk[1];
                        for x in 0..width {
                            let value =
                                (current_scanline[(x / 4) as usize] >> (3 - x % 4) * 2) & 0x3;
                            *retval.get_mut(x, y) = math::Vec4::new(
                                value * 0x55,
                                value * 0x55,
                                value * 0x55,
                                if value == transparent_value { 0 } else { 0xFF },
                            );
                        }
                    }
                    (4, ColorType::Grayscale, None) => {
                        for x in 0..width {
                            let value =
                                (current_scanline[(x / 2) as usize] >> (1 - x % 2) * 4) & 0xF;
                            *retval.get_mut(x, y) =
                                math::Vec4::new(value * 0x11, value * 0x11, value * 0x11, 0xFF);
                        }
                    }
                    (4, ColorType::Grayscale, Some(trns_chunk)) => {
                        let transparent_value = trns_chunk[1];
                        for x in 0..width {
                            let value =
                                (current_scanline[(x / 2) as usize] >> (1 - x % 2) * 4) & 0xF;
                            *retval.get_mut(x, y) = math::Vec4::new(
                                value * 0x11,
                                value * 0x11,
                                value * 0x11,
                                if value == transparent_value { 0 } else { 0xFF },
                            );
                        }
                    }
                    (8, ColorType::Grayscale, None) => {
                        for x in 0..width {
                            let value = current_scanline[x as usize];
                            *retval.get_mut(x, y) = math::Vec4::new(value, value, value, 0xFF);
                        }
                    }
                    (8, ColorType::Grayscale, Some(trns_chunk)) => {
                        let transparent_value = trns_chunk[1];
                        for x in 0..width {
                            let value = current_scanline[x as usize];
                            *retval.get_mut(x, y) = math::Vec4::new(
                                value,
                                value,
                                value,
                                if value == transparent_value { 0 } else { 0xFF },
                            );
                        }
                    }
                    (16, ColorType::Grayscale, None) => {
                        for x in 0..width {
                            let value = u16_to_u8(
                                ((current_scanline[x as usize * 2] as u16) << 8)
                                    | current_scanline[x as usize * 2 + 1] as u16,
                            );
                            *retval.get_mut(x, y) = math::Vec4::new(value, value, value, 0xFF);
                        }
                    }
                    (16, ColorType::Grayscale, Some(trns_chunk)) => {
                        let transparent_value =
                            ((trns_chunk[0] as u16) << 8) | trns_chunk[1] as u16;
                        for x in 0..width {
                            let value = ((current_scanline[x as usize * 2] as u16) << 8)
                                | current_scanline[x as usize * 2 + 1] as u16;
                            let a = if value == transparent_value { 0 } else { 0xFF };
                            let value = u16_to_u8(value);
                            *retval.get_mut(x, y) = math::Vec4::new(value, value, value, a);
                        }
                    }
                    (8, ColorType::RGB, None) => {
                        for x in 0..width {
                            *retval.get_mut(x, y) = math::Vec4::new(
                                current_scanline[x as usize * 3],
                                current_scanline[x as usize * 3 + 1],
                                current_scanline[x as usize * 3 + 2],
                                0xFF,
                            );
                        }
                    }
                    (8, ColorType::RGB, Some(trns_chunk)) => {
                        let transparent_r = trns_chunk[1];
                        let transparent_g = trns_chunk[3];
                        let transparent_b = trns_chunk[5];
                        for x in 0..width {
                            let r = current_scanline[x as usize * 3];
                            let g = current_scanline[x as usize * 3 + 1];
                            let b = current_scanline[x as usize * 3 + 2];
                            *retval.get_mut(x, y) = math::Vec4::new(
                                r,
                                g,
                                b,
                                if r == transparent_r && g == transparent_g && b == transparent_b {
                                    0
                                } else {
                                    0xFF
                                },
                            );
                        }
                    }
                    (16, ColorType::RGB, None) => {
                        for x in 0..width {
                            let r = u16_to_u8(
                                ((current_scanline[x as usize * 6] as u16) << 8)
                                    | current_scanline[x as usize * 6 + 1] as u16,
                            );
                            let g = u16_to_u8(
                                ((current_scanline[x as usize * 6 + 2] as u16) << 8)
                                    | current_scanline[x as usize * 6 + 3] as u16,
                            );
                            let b = u16_to_u8(
                                ((current_scanline[x as usize * 6 + 4] as u16) << 8)
                                    | current_scanline[x as usize * 6 + 5] as u16,
                            );
                            *retval.get_mut(x, y) = math::Vec4::new(r, g, b, 0xFF);
                        }
                    }
                    (16, ColorType::RGB, Some(trns_chunk)) => {
                        let transparent_r = ((trns_chunk[0] as u16) << 8) | trns_chunk[1] as u16;
                        let transparent_g = ((trns_chunk[2] as u16) << 8) | trns_chunk[3] as u16;
                        let transparent_b = ((trns_chunk[4] as u16) << 8) | trns_chunk[5] as u16;
                        for x in 0..width {
                            let r = ((current_scanline[x as usize * 6] as u16) << 8)
                                | current_scanline[x as usize * 6 + 1] as u16;
                            let g = ((current_scanline[x as usize * 6 + 2] as u16) << 8)
                                | current_scanline[x as usize * 6 + 3] as u16;
                            let b = ((current_scanline[x as usize * 6 + 4] as u16) << 8)
                                | current_scanline[x as usize * 6 + 5] as u16;
                            let a =
                                if r == transparent_r && g == transparent_g && b == transparent_b {
                                    0
                                } else {
                                    0xFF
                                };
                            let r = u16_to_u8(r);
                            let g = u16_to_u8(g);
                            let b = u16_to_u8(b);
                            *retval.get_mut(x, y) = math::Vec4::new(r, g, b, a);
                        }
                    }
                    (1, ColorType::RGBPalette, None) => {
                        let palette = palette.as_ref().unwrap();
                        for x in 0..width {
                            let index = (current_scanline[(x / 8) as usize] >> (7 - x % 8)) & 0x1;
                            let index = index as usize * 3;
                            if index >= palette.len() {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "pixel palette index out of range",
                                ));
                            }
                            *retval.get_mut(x, y) = math::Vec4::new(
                                palette[index],
                                palette[index + 1],
                                palette[index + 2],
                                0xFF,
                            );
                        }
                    }
                    (1, ColorType::RGBPalette, Some(trns_chunk)) => {
                        let palette = palette.as_ref().unwrap();
                        for x in 0..width {
                            let index = ((current_scanline[(x / 8) as usize] >> (7 - x % 8)) & 0x1)
                                as usize;
                            if index * 3 >= palette.len() {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "pixel palette index out of range",
                                ));
                            }
                            *retval.get_mut(x, y) = math::Vec4::new(
                                palette[index * 3],
                                palette[index * 3 + 1],
                                palette[index * 3 + 2],
                                trns_chunk[index],
                            );
                        }
                    }
                    (2, ColorType::RGBPalette, None) => {
                        let palette = palette.as_ref().unwrap();
                        for x in 0..width {
                            let index =
                                (current_scanline[(x / 4) as usize] >> (3 - x % 4) * 2) & 0x3;
                            let index = index as usize * 3;
                            if index >= palette.len() {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "pixel palette index out of range",
                                ));
                            }
                            *retval.get_mut(x, y) = math::Vec4::new(
                                palette[index],
                                palette[index + 1],
                                palette[index + 2],
                                0xFF,
                            );
                        }
                    }
                    (2, ColorType::RGBPalette, Some(trns_chunk)) => {
                        let palette = palette.as_ref().unwrap();
                        for x in 0..width {
                            let index = ((current_scanline[(x / 4) as usize] >> (3 - x % 4) * 2)
                                & 0x3) as usize;
                            if index * 3 >= palette.len() {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "pixel palette index out of range",
                                ));
                            }
                            *retval.get_mut(x, y) = math::Vec4::new(
                                palette[index * 3],
                                palette[index * 3 + 1],
                                palette[index * 3 + 2],
                                trns_chunk[index],
                            );
                        }
                    }
                    (4, ColorType::RGBPalette, None) => {
                        let palette = palette.as_ref().unwrap();
                        for x in 0..width {
                            let index =
                                (current_scanline[(x / 2) as usize] >> (1 - x % 2) * 4) & 0xF;
                            let index = index as usize * 3;
                            if index >= palette.len() {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "pixel palette index out of range",
                                ));
                            }
                            *retval.get_mut(x, y) = math::Vec4::new(
                                palette[index],
                                palette[index + 1],
                                palette[index + 2],
                                0xFF,
                            );
                        }
                    }
                    (4, ColorType::RGBPalette, Some(trns_chunk)) => {
                        let palette = palette.as_ref().unwrap();
                        for x in 0..width {
                            let index = ((current_scanline[(x / 2) as usize] >> (1 - x % 2) * 4)
                                & 0xF) as usize;
                            if index * 3 >= palette.len() {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "pixel palette index out of range",
                                ));
                            }
                            *retval.get_mut(x, y) = math::Vec4::new(
                                palette[index * 3],
                                palette[index * 3 + 1],
                                palette[index * 3 + 2],
                                trns_chunk[index],
                            );
                        }
                    }
                    (8, ColorType::RGBPalette, None) => {
                        let palette = palette.as_ref().unwrap();
                        for x in 0..width {
                            let index = current_scanline[x as usize];
                            let index = index as usize * 3;
                            if index >= palette.len() {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "pixel palette index out of range",
                                ));
                            }
                            *retval.get_mut(x, y) = math::Vec4::new(
                                palette[index],
                                palette[index + 1],
                                palette[index + 2],
                                0xFF,
                            );
                        }
                    }
                    (8, ColorType::RGBPalette, Some(trns_chunk)) => {
                        let palette = palette.as_ref().unwrap();
                        for x in 0..width {
                            let index = current_scanline[x as usize] as usize;
                            if index * 3 >= palette.len() {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "pixel palette index out of range",
                                ));
                            }
                            *retval.get_mut(x, y) = math::Vec4::new(
                                palette[index * 3],
                                palette[index * 3 + 1],
                                palette[index * 3 + 2],
                                trns_chunk[index],
                            );
                        }
                    }
                    (8, ColorType::GrayscaleAlpha, _) => {
                        for x in 0..width {
                            *retval.get_mut(x, y) = math::Vec4::new(
                                current_scanline[x as usize * 2],
                                current_scanline[x as usize * 2],
                                current_scanline[x as usize * 2],
                                current_scanline[x as usize * 2 + 1],
                            );
                        }
                    }
                    (16, ColorType::GrayscaleAlpha, _) => {
                        for x in 0..width {
                            let v = u16_to_u8(
                                ((current_scanline[x as usize * 4] as u16) << 8)
                                    | current_scanline[x as usize * 4 + 1] as u16,
                            );
                            let a = u16_to_u8(
                                ((current_scanline[x as usize * 4 + 2] as u16) << 8)
                                    | current_scanline[x as usize * 4 + 3] as u16,
                            );
                            *retval.get_mut(x, y) = math::Vec4::new(v, v, v, a);
                        }
                    }
                    (8, ColorType::RGBA, _) => {
                        for x in 0..width {
                            *retval.get_mut(x, y) = math::Vec4::new(
                                current_scanline[x as usize * 4],
                                current_scanline[x as usize * 4 + 1],
                                current_scanline[x as usize * 4 + 2],
                                current_scanline[x as usize * 4 + 3],
                            );
                        }
                    }
                    (16, ColorType::RGBA, _) => {
                        for x in 0..width {
                            let r = u16_to_u8(
                                ((current_scanline[x as usize * 8] as u16) << 8)
                                    | current_scanline[x as usize * 8 + 1] as u16,
                            );
                            let g = u16_to_u8(
                                ((current_scanline[x as usize * 8 + 2] as u16) << 8)
                                    | current_scanline[x as usize * 8 + 3] as u16,
                            );
                            let b = u16_to_u8(
                                ((current_scanline[x as usize * 8 + 4] as u16) << 8)
                                    | current_scanline[x as usize * 8 + 5] as u16,
                            );
                            let a = u16_to_u8(
                                ((current_scanline[x as usize * 8 + 6] as u16) << 8)
                                    | current_scanline[x as usize * 8 + 7] as u16,
                            );
                            *retval.get_mut(x, y) = math::Vec4::new(r, g, b, a);
                        }
                    }
                    _ => unimplemented!(),
                }
                mem::swap(&mut current_scanline, &mut last_scanline);
            }
            Ok(Some(retval))
        }
        let retval = {
            let compressed_byte_reader = CompressedByteReader {
                chunk_iterator: &mut chunk_iterator,
                chunk: None,
            };
            let mut decompressed_reader =
                DeflateDecoder::from_zlib(ChunkingReader::new(compressed_byte_reader));
            let IHDRChunk {
                width,
                height,
                bit_depth,
                color_type,
                compression_method,
                filter_method,
                interlace_method,
            } = ihdr_chunk;
            let retval = match (compression_method, filter_method, interlace_method) {
                (
                    CompressionMethod::Deflate,
                    FilterMethod::Adaptive,
                    InterlaceMethod::NoInterlace,
                ) => decode_image(
                    width,
                    height,
                    bit_depth,
                    color_type,
                    &palette,
                    &trns_chunk,
                    &mut decompressed_reader,
                )?.unwrap(),
                (CompressionMethod::Deflate, FilterMethod::Adaptive, InterlaceMethod::Adam7) => {
                    let mut retval = Image::new(width, height, Default::default());
                    {
                        let mut decode_subpass = |width_divisor: u32,
                                                  height_divisor: u32,
                                                  width_offset: u32,
                                                  height_offset: u32|
                         -> Result<()> {
                            if let Some(subpass) = decode_image(
                                width / width_divisor + if width % width_divisor <= width_offset {
                                    0
                                } else {
                                    1
                                },
                                height / height_divisor + if height % height_divisor
                                    <= height_offset
                                {
                                    0
                                } else {
                                    1
                                },
                                bit_depth,
                                color_type,
                                &palette,
                                &trns_chunk,
                                &mut decompressed_reader,
                            )? {
                                for y in 0..subpass.height() {
                                    for x in 0..subpass.width() {
                                        *retval.get_mut(
                                            x * width_divisor + width_offset,
                                            y * height_divisor + height_offset,
                                        ) = *subpass.get(x, y);
                                    }
                                }
                            }
                            Ok(())
                        };
                        decode_subpass(8, 8, 0, 0)?;
                        decode_subpass(8, 8, 4, 0)?;
                        decode_subpass(4, 8, 0, 4)?;
                        decode_subpass(4, 4, 2, 0)?;
                        decode_subpass(2, 4, 0, 2)?;
                        decode_subpass(2, 2, 1, 0)?;
                        decode_subpass(1, 2, 0, 1)?;
                    }
                    retval
                }
            };
            match decompressed_reader.bytes().next() {
                Some(Err(error)) => return Err(error),
                Some(Ok(_)) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "compressed data is too long",
                    ))
                }
                None => {}
            }
            retval
        };
        loop {
            if let Some(Ok(Chunk {
                name: ChunkName::IEND,
                ..
            })) = chunk_iterator.peek()
            {
                break;
            }
            handle_ignored_chunk(
                chunk_iterator
                    .next()
                    .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing IEND chunk"))??,
            )?;
        }
        if chunk_iterator
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "missing IEND chunk"))??
            .data
            .len()
            != 0
        {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "IEND chunk is not zero size",
            ));
        }
        if let Some(next) = chunk_iterator.next() {
            let chunk = next?;
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("chunk after IEND chunk: {:?}", chunk.name),
            ));
        }
        Ok(retval)
    }
}

pub fn get_image_loader() -> PngImageLoader {
    Default::default()
}
