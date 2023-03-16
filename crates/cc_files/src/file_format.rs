use std::ffi::OsStr;

// TODO: use infer
static MAGIC_BYTES: [(&[u8], FileFormat); 7] = [
    (b"\x89PNG\r\n\x1a\n", FileFormat::Png),
    (&[0xff, 0xd8, 0xff], FileFormat::Jpeg),
    (b"GIF89a", FileFormat::Gif),
    (b"GIF87a", FileFormat::Gif),
    (b"RIFF", FileFormat::WebP),
    (&[0x3c, 0x3f, 0x78, 0x6d, 0x6c], FileFormat::Svg),
    (&[0x3c, 0x73, 0x76, 0x67, 0x20], FileFormat::Svg),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[non_exhaustive]
pub enum FileFormat {
    /// An Image in PNG Format
    Png,

    /// An Image in JPEG Format
    Jpeg,

    /// An Image in GIF Format
    Gif,

    /// An Image in WEBP Format
    WebP,

    /// An Image in Svg Format
    Svg,
}

impl FileFormat {
    #[inline]
    pub fn from_extension<S>(ext: S) -> Option<Self>
    where
        S: AsRef<OsStr>,
    {
        // thin wrapper function to strip generics
        fn inner(ext: &OsStr) -> Option<FileFormat> {
            let ext = ext.to_str()?.to_ascii_lowercase();

            Some(match ext.as_str() {
                "jpg" | "jpeg" => FileFormat::Jpeg,
                "png" => FileFormat::Png,
                "gif" => FileFormat::Gif,
                "webp" => FileFormat::WebP,
                "svg" => FileFormat::Svg,
                _ => return None,
            })
        }

        inner(ext.as_ref())
    }

    pub fn from_mime_type<M>(mime_type: M) -> Option<Self>
    where
        M: AsRef<str>,
    {
        match mime_type.as_ref() {
            "image/jpeg" => Some(FileFormat::Jpeg),
            "image/png" => Some(FileFormat::Png),
            "image/gif" => Some(FileFormat::Gif),
            "image/webp" => Some(FileFormat::WebP),
            "image/svg+xml" => Some(FileFormat::Svg),
            _ => None,
        }
    }
}

pub(crate) fn guess_format(buffer: &[u8]) -> Option<FileFormat> {
    for &(signature, format) in &MAGIC_BYTES {
        if buffer.starts_with(signature) {
            return Some(format);
        }
    }

    None
}
