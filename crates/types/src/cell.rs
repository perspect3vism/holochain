//! A "Cell" represents a DNA/AgentId pair - a space where one dna/agent
//! can track its source chain and service network requests / responses.

use derive_more::{Display, From, Into};
use fixt::prelude::*;
use holo_hash::{AgentPubKey, AgentPubKeyFixturator, DnaHash, DnaHashFixturator};
use std::fmt;

/// The unique identifier for a Cell.
/// Cells are uniquely determined by this pair - this pair is necessary
/// and sufficient to refer to a cell in a conductor
#[derive(Clone, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CellId(DnaHash, AgentPubKey);

fixturator!(
    CellId,
    CellId(
        DnaHashFixturator::new(Empty).next().unwrap(),
        AgentPubKeyFixturator::new(Empty).next().unwrap()
    ),
    CellId(
        DnaHashFixturator::new(Unpredictable).next().unwrap(),
        AgentPubKeyFixturator::new(Unpredictable).next().unwrap()
    ),
    {
        let ret = CellId(
            DnaHashFixturator::new_indexed(Predictable, self.0.index)
                .next()
                .unwrap(),
            AgentPubKeyFixturator::new_indexed(Predictable, self.0.index)
                .next()
                .unwrap(),
        );
        self.0.index = self.0.index + 1;
        ret
    }
);

/// A conductor-specific name for a Cell
/// (Used to be instance_id)
#[derive(
    Clone, Debug, Display, Hash, PartialEq, Eq, From, Into, serde::Serialize, serde::Deserialize,
)]
pub struct CellHandle(String);

impl From<&str> for CellHandle {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl fmt::Display for CellId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cell-{}-{}", self.dna_hash(), self.agent_pubkey())
    }
}

impl CellId {
    /// The dna hash/address for this cell.
    pub fn dna_hash(&self) -> &DnaHash {
        &self.0
    }

    /// The agent id / public key for this cell.
    pub fn agent_pubkey(&self) -> &AgentPubKey {
        &self.1
    }
}

impl From<(DnaHash, AgentPubKey)> for CellId {
    fn from(pair: (DnaHash, AgentPubKey)) -> Self {
        Self(pair.0, pair.1)
    }
}
