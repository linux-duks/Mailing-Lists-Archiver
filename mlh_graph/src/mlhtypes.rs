use std::{collections::HashSet, str::FromStr};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Object type of an MLHID
pub enum NodeType {
    EmailBody = 0,
    EmailMessage = 1,
    Origin = 2,
    MailingList = 3,
    Patch = 4,
    Person = 5,
}

impl<'a> TryFrom<&'a [u8]> for NodeType {
    type Error = &'a [u8];
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(match value {
            b"emb" => Self::EmailBody,
            b"emm" => Self::EmailMessage,
            b"ori" => Self::Origin,
            b"mls" => Self::MailingList,
            b"ptc" => Self::Patch,
            b"prs" => Self::Person,
            _ => return Err(value),
        })
    }
}

impl FromStr for NodeType {
    type Err = String;

    /// # Examples
    ///
    /// ```
    /// # use mlh_graph::NodeType;
    ///
    /// assert_eq!("emm".parse::<NodeType>(), Ok(NodeType::EmailMessage));
    /// assert!(matches!("xyz".parse::<NodeType>(), Err(_)));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "emb" => Self::EmailBody,
            "emm" => Self::EmailMessage,
            "ori" => Self::Origin,
            "mls" => Self::MailingList,
            "ptc" => Self::Patch,
            "prs" => Self::Person,
            _ => return Err(s.to_owned()),
        })
    }
}

impl TryFrom<u8> for NodeType {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::EmailBody,
            1 => Self::EmailMessage,
            2 => Self::Origin,
            3 => Self::MailingList,
            4 => Self::Patch,
            5 => Self::Person,
            _ => return Err(value),
        })
    }
}

impl NodeType {
    /// Get the number of possible types.
    ///
    /// To avoid having to update this when adding a new type
    /// we can use the unstable function `std::mem::variant_count`
    /// or the `variant_count` crate.
    /// But for now we just hardcode it while we decide how to
    /// deal with this.
    pub const NUMBER_OF_TYPES: usize = 6;

    /// The number of bits needed to store the node type as integers
    /// This is `ceil(log2(NUMBER_OF_TYPES))`  which can be arithmetized into
    /// `floor(log2(NUMBER_OF_TYPES))` plus one if it's not a power of two.
    pub const BITWIDTH: usize = Self::NUMBER_OF_TYPES.ilog2() as usize
        + (!Self::NUMBER_OF_TYPES.is_power_of_two()) as usize;

    /// Convert a type to the str used in the SWHID
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::EmailBody => "emb",
            Self::EmailMessage => "emm",
            Self::Origin => "ori",
            Self::MailingList => "mls",
            Self::Patch => "ptc",
            Self::Person => "prs",
        }
    }

    /// Convert a type to its enum discriminant value.
    ///
    /// In all cases using this method is both safer and more concise than
    /// `(node_type as isize).try_into().unwrap()`.
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::EmailBody => 0,
            Self::EmailMessage => 1,
            Self::Origin => 2,
            Self::MailingList => 3,
            Self::Patch => 4,
            Self::Person => 5,
        }
    }

    /// Returns a vector containing all possible `NodeType` values.
    pub fn all() -> HashSet<Self> {
        let mut hst = HashSet::new();
        hst.insert(NodeType::EmailBody);
        hst.insert(NodeType::EmailMessage);
        hst.insert(NodeType::Origin);
        hst.insert(NodeType::MailingList);
        hst.insert(NodeType::Patch);
        hst.insert(NodeType::Person);
        hst
    }
}

impl core::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

/// Compact representation of a set of [NodeType]-s, as a bit array.
type NodeTypeSet = u64;

/// Constraint on allowed node types, as a set of node types.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NodeConstraint(pub NodeTypeSet);

impl Default for NodeConstraint {
    fn default() -> Self {
        Self(0b111111)
    }
}

impl NodeConstraint {
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use mlh_graph::{NodeConstraint, NodeType};
    ///
    /// let only_mails: NodeConstraint = "emm".parse().unwrap();
    /// let all_nodes: NodeConstraint = "*".parse().unwrap();
    ///
    /// assert!(only_mails.matches(NodeType::EmailMessage));
    /// assert!(!only_mails.matches(NodeType::EmailBody));
    /// for node_type in NodeType::all() {
    ///     assert!(all_nodes.matches(node_type));
    /// }
    /// ```
    pub fn matches(&self, node_type: NodeType) -> bool {
        self.0 & (1 << node_type.to_u8()) != 0
    }

    pub fn to_vec(&self) -> Vec<NodeType> {
        (0..NodeType::NUMBER_OF_TYPES as u8)
            .filter(|type_idx| self.0 & (1 << type_idx) != 0)
            .map(|type_idx| type_idx.try_into().unwrap())
            .collect()
    }
}

impl FromStr for NodeConstraint {
    type Err = String;

    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// # use mlh_graph::{NodeConstraint, NodeType};
    ///
    /// assert_eq!("*".parse::<NodeConstraint>(), Ok(NodeConstraint(0b111111)));
    /// assert_eq!("emm".parse::<NodeConstraint>(), Ok(NodeConstraint(0b000010)));
    /// assert_eq!("emm,emb".parse::<NodeConstraint>(), Ok(NodeConstraint(0b000011)));
    /// assert!(matches!("xyz".parse::<NodeConstraint>(), Err(_)));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "*" {
            Ok(NodeConstraint::default())
        } else {
            let mut node_types = 0;
            for s in s.split(',') {
                node_types |= 1 << s.parse::<NodeType>()?.to_u8();
            }
            Ok(NodeConstraint(node_types))
        }
    }
}

impl core::fmt::Display for NodeConstraint {
    /// ```
    /// # use std::collections::HashSet;
    /// # use mlh_graph::{NodeConstraint, NodeType};
    ///
    /// assert_eq!(format!("{}", NodeConstraint::default()), "*");
    /// assert_eq!(
    ///     format!("{}", NodeConstraint(0b000011)),
    ///     "emb,emm"
    /// );
    /// assert_eq!(
    ///     format!("{}", NodeConstraint(0b111100)),
    ///     "ptc,mls,ori"
    /// );
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == Self::default() {
            write!(f, "*")?;
        } else {
            let mut type_strings: Vec<&str> = self.to_vec().iter().map(|t| t.to_str()).collect();
            type_strings.sort();
            write!(f, "{}", type_strings.join(","))?;
        }
        Ok(())
    }
}

/// Type of an arc between two nodes in the Software Heritage graph, as a pair
/// of type constraints on the source and destination arc. When one of the two
/// is None, it means "any node type accepted".
// TODO remove Options from ArcType and create a (more  expressive, similar to
// NodeConstraint) type called ArcConstraint
pub struct ArcType {
    pub src: Option<NodeType>,
    pub dst: Option<NodeType>,
}
