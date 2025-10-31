use std::str::FromStr;

use rdst::RadixKey;
use sha1::{Digest, Sha1};
use thiserror::Error;

use crate::NodeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct MLHID {
    /// Namespace Version
    pub namespace_version: u8,
    /// Node type
    pub node_type: NodeType,
    /// SHA1 has of the node
    pub hash: [u8; 20],
}

impl MLHID {
    /// The size of the binary representation of a MLHID
    pub const BYTES_SIZE: usize = 22;

    /// Returns the pseudo-MLHID representation for a origin URI
    /// akin to "mlh:1:prs:{}"
    pub fn from_person_identification(identification: impl AsRef<str>) -> MLHID {
        let mut hasher = Sha1::new();
        hasher.update(identification.as_ref());

        MLHID {
            namespace_version: 1,
            node_type: NodeType::Person,
            hash: hasher.finalize().into(),
        }
    }

    /// Returns the pseudo-MLHID representation for a origin URI
    /// akin to "mlh:1:ori:{}"
    pub fn from_origin_url(origin: impl AsRef<str>) -> MLHID {
        let mut hasher = Sha1::new();
        hasher.update(origin.as_ref());

        MLHID {
            namespace_version: 1,
            node_type: NodeType::Origin,
            hash: hasher.finalize().into(),
        }
    }

    /// Returns the pseudo-MLHID representation for a str
    /// akin to "mlh:1:{}:{}"
    pub fn from_content_str(node_type: NodeType, content: impl AsRef<str>) -> MLHID {
        let mut hasher = Sha1::new();
        hasher.update(content.as_ref());

        MLHID {
            namespace_version: 1,
            node_type,
            hash: hasher.finalize().into(),
        }
    }
}

impl core::fmt::Display for MLHID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "mlh:{}:{}:",
            self.namespace_version,
            self.node_type.to_str(),
        )?;
        for byte in self.hash.iter() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum BinMLHIDDeserializationError {
    #[error("Unsupported MLHID version: {0}")]
    Version(u8),
    #[error("Invalid MLHID type: {0}")]
    Type(u8),
}

/// Parse a MLHID from bytes, while the MLHID struct has the exact same layout
/// and thus it can be read directly from bytes, this function is provided for
/// completeness and safety because we can check the namespace version is
/// supported.
impl TryFrom<[u8; MLHID::BYTES_SIZE]> for MLHID {
    type Error = BinMLHIDDeserializationError;
    fn try_from(value: [u8; MLHID::BYTES_SIZE]) -> std::result::Result<Self, Self::Error> {
        use BinMLHIDDeserializationError::*;

        let namespace_version = value[0];
        if namespace_version != 1 {
            return Err(Version(namespace_version));
        }
        let node_type = NodeType::try_from(value[1]).map_err(Type)?;
        let mut hash = [0; 20];
        hash.copy_from_slice(&value[2..]);
        Ok(Self {
            namespace_version,
            node_type,
            hash,
        })
    }
}

#[derive(Error, Debug, PartialEq, Eq, Hash)]
pub enum StrMLHIDDeserializationError {
    #[error("Invalid syntax: {0}")]
    Syntax(&'static str),
    #[error("Unsupported MLHID namespace: {0}")]
    Namespace(String),
    #[error("Unsupported MLHID version: {0}")]
    Version(String),
    #[error("Expected hash length to be {expected}, got {got}")]
    HashLength { expected: usize, got: usize },
    #[error("Invalid MLHID type: {0}")]
    Type(String),
    #[error("MLHID hash is not hexadecimal: {0}")]
    HashAlphabet(String),
}

/// Parse a MLHID from the string representation
impl TryFrom<&str> for MLHID {
    type Error = StrMLHIDDeserializationError;
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl FromStr for MLHID {
    type Err = StrMLHIDDeserializationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        use StrMLHIDDeserializationError::*;

        let mut tokens = value.splitn(4, ':');
        let Some(namespace) = tokens.next() else {
            return Err(Syntax("MLHID is empty"));
        };
        if namespace != "mlh" {
            return Err(Namespace(namespace.to_string()));
        }
        let Some(namespace_version) = tokens.next() else {
            return Err(Syntax("MLHID is too short (no namespace version)"));
        };
        if namespace_version != "1" {
            return Err(Version(namespace_version.to_string()));
        }
        let Some(node_type) = tokens.next() else {
            return Err(Syntax("MLHID is too short (no object type)"));
        };
        let Some(hex_hash) = tokens.next() else {
            return Err(Syntax("MLHID is too short (no object hash)"));
        };
        if hex_hash.len() != 40 {
            return Err(HashLength {
                expected: 40,
                got: hex_hash.len(),
            });
        }
        let node_type = node_type
            .parse::<NodeType>()
            .map_err(|e| Type(e.to_string()))?;
        let mut hash = [0u8; 20];

        faster_hex::hex_decode(hex_hash.as_bytes(), &mut hash)
            .map_err(|_| HashAlphabet(hex_hash.to_string()))?;

        Ok(Self {
            namespace_version: 1,
            node_type,
            hash,
        })
    }
}

impl From<MLHID> for [u8; MLHID::BYTES_SIZE] {
    fn from(value: MLHID) -> Self {
        let mut result = [0; MLHID::BYTES_SIZE];
        result[0] = value.namespace_version;
        result[1] = value.node_type as u8;
        result[2..].copy_from_slice(&value.hash);
        result
    }
}

impl RadixKey for MLHID {
    const LEVELS: usize = 22;

    #[inline(always)]
    fn get_level(&self, level: usize) -> u8 {
        assert!(level < Self::LEVELS);
        match Self::LEVELS - level - 1 {
            0 => self.namespace_version,
            1 => match self.node_type {
                // must follow alphabetical order of the 3-char abbreviation
                NodeType::EmailBody => 0,    // emb
                NodeType::EmailMessage => 1, // emm
                NodeType::MailingList => 2,  // mls
                NodeType::Origin => 3,       // ori
                NodeType::Patch => 4,        // ptc
                NodeType::Person => 5,       // prs
            },
            n => self.hash[n - 2],
        }
    }
}

impl Ord for MLHID {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for level in (0..Self::LEVELS).rev() {
            let ordering = self.get_level(level).cmp(&other.get_level(level));
            if ordering != std::cmp::Ordering::Equal {
                return ordering;
            }
        }
        std::cmp::Ordering::Equal
    }
}
impl PartialOrd for MLHID {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl serde::Serialize for MLHID {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for MLHID {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        deserializer.deserialize_str(MLHIDVisitor)
    }
}

struct MLHIDVisitor;

impl serde::de::Visitor<'_> for MLHIDVisitor {
    type Value = MLHID;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a MLHID")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        value.try_into().map_err(E::custom)
    }
}

/// Helper function for [`mlhid!()`]
pub const fn __parse_mlhid(node_type: crate::NodeType, hash: &'static str) -> MLHID {
    use const_panic::unwrap_ok;
    unwrap_ok!(match const_hex::const_decode_to_array(hash.as_bytes()) {
        Ok(hash) => Ok(MLHID {
            namespace_version: 1,
            node_type,
            hash
        }),
        Err(_) => Err("invalid MLHID hash"),
    })
}

/// A MLHID literal checked at compile time
///
/// # Examples
///
/// ```
/// use mlh_graph::mlhid;
/// assert_eq!(
///     mlhid!(mlh:1:emb:0000000000000000000000000000000000000004).to_string(),
///     "mlh:1:emb:0000000000000000000000000000000000000004".to_string(),
/// );
/// assert_eq!(
///     mlhid!(mlh:1:emm:ffffffffffffffffffffffffffff000000000004).to_string(),
///     "mlh:1:emm:ffffffffffffffffffffffffffff000000000004".to_string(),
/// );
/// assert_eq!(
///     mlhid!(mlh:1:mls:FFFFFFFFFFFFFFFFFFFFFFFFFFFF000000000004).to_string(),
///     "mlh:1:mls:ffffffffffffffffffffffffffff000000000004".to_string(),
/// );
/// ```
///
/// ```compile_fail
/// use mlh_graph::mlhid;
/// mlhid!(mlh:1:rev:ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ);
/// ```
///
/// ```compile_fail
/// use mlh_graph::mlhid;
/// mlhid!(mlh:1:rev:00000000000000000000000000000000000004);
/// ```
#[macro_export]
macro_rules! mlhid {
    // hash starting with a decimal digit
    (mlh:1:cnt:$hash:literal) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Content, hash)
        };
        mlhid
    }};
    (mlh:1:dir:$hash:literal) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Directory, hash)
        };
        mlhid
    }};
    (mlh:1:rev:$hash:literal) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Revision, hash)
        };
        mlhid
    }};
    (mlh:1:rel:$hash:literal) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Release, hash)
        };
        mlhid
    }};
    (mlh:1:snp:$hash:literal) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Snapshot, hash)
        };
        mlhid
    }};
    (mlh:1:ori:$hash:literal) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Origin, hash)
        };
        mlhid
    }};

    // hash starting with a to f
    (mlh:1:cnt:$hash:ident) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Content, hash)
        };
        mlhid
    }};
    (mlh:1:dir:$hash:ident) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Directory, hash)
        };
        mlhid
    }};
    (mlh:1:rev:$hash:ident) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Revision, hash)
        };
        mlhid
    }};
    (mlh:1:rel:$hash:ident) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Release, hash)
        };
        mlhid
    }};
    (mlh:1:snp:$hash:ident) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Snapshot, hash)
        };
        mlhid
    }};
    (mlh:1:ori:$hash:ident) => {{
        const mlhid: ::mlh_graph::MLHID = {
            let hash: &str = stringify!($hash);
            ::mlh_graph::__parse_mlhid(::mlh_graph::NodeType::Origin, hash)
        };
        mlhid
    }};
}
