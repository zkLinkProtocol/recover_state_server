use serde::{Deserialize, Serialize};

use zklink_basic_types::U256;

use crate::bellman::plonk::better_cs::{
    cs::PlonkCsWidth4WithNextStepParams, keys::Proof as OldProof,
};
use crate::serialization::{serialize_single_proof, SingleProofSerde};
use crate::Engine;

pub type OldProofType = OldProof<Engine, PlonkCsWidth4WithNextStepParams>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleProof(#[serde(with = "SingleProofSerde")] pub OldProofType);

impl From<OldProofType> for SingleProof {
    fn from(proof: OldProofType) -> Self {
        SingleProof(proof)
    }
}

impl Default for SingleProof {
    fn default() -> Self {
        SingleProof(OldProofType::empty())
    }
}

impl SingleProof {
    pub fn serialize_single_proof(&self) -> EncodedSingleProof {
        serialize_single_proof(&self.0)
    }
}

/// Encoded representation of the block proof.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EncodedSingleProof {
    pub inputs: Vec<U256>,
    pub proof: Vec<U256>,
}

impl Default for EncodedSingleProof {
    fn default() -> Self {
        Self {
            inputs: vec![U256::default(); 1],
            proof: vec![U256::default(); 33],
        }
    }
}
