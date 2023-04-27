// Built-in uses
use std::convert::TryFrom;
// External uses
// Workspace uses
use zklink_types::operations::ZkLinkOp;
// Local uses
use super::v0;
use crate::contract::utils;
use crate::rollup_ops::RollupOpsBlock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZkLinkContractVersion {
    V0,
}

impl TryFrom<u32> for ZkLinkContractVersion {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use ZkLinkContractVersion::*;

        match value {
            0 => Ok(V0),
            _ => Err(anyhow::anyhow!("Unsupported contract version")),
        }
    }
}
impl From<ZkLinkContractVersion> for i16 {
    fn from(val: ZkLinkContractVersion) -> Self {
        match val {
            ZkLinkContractVersion::V0 => 0,
        }
    }
}

impl ZkLinkContractVersion {
    pub fn rollup_ops_blocks_from_bytes(
        &self,
        data: Vec<u8>,
    ) -> anyhow::Result<Vec<RollupOpsBlock>> {
        use ZkLinkContractVersion::*;
        let mut blocks = match self {
            V0 => v0::rollup_ops_blocks_from_bytes(data)?,
        };
        // Set the contract version.
        for block in blocks.iter_mut() {
            block.contract_version = Some(*self);
        }
        Ok(blocks)
    }

    /// Attempts to restore block operations from the public data
    /// committed on the Layer1 smart contract.
    ///
    /// # Arguments
    ///
    /// * `data` - public data for block operations
    ///
    pub fn get_rollup_ops_from_data(&self, data: &[u8]) -> Result<Vec<ZkLinkOp>, anyhow::Error> {
        use ZkLinkContractVersion::*;
        match self {
            V0 => utils::get_rollup_ops_from_data(data),
        }
    }

    /// Returns the contract version incremented by `num`.
    ///
    /// # Arguments
    ///
    /// * `num` - how many times to upgrade.
    ///
    /// # Panics
    ///
    /// Panics if the the result is greater than the latest supported version.
    pub fn upgrade(&self, num: u32) -> Self {
        Self::try_from(i16::from(*self) as u32 + num)
            .expect("cannot upgrade past the latest contract version")
    }

    pub fn supported_ops_numbers(&self) -> &'static [usize] {
        use ZkLinkContractVersion::*;
        match self {
            V0 => &[111, 401, 511],
        }
    }
}
