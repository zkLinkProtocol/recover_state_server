use crate::exit_circuit::*;
use crate::witness::AccountWitness;

pub struct AccountContent<E: RescueEngine> {
    pub nonce: CircuitElement<E>,
    pub pub_key_hash: CircuitElement<E>,
    pub address: CircuitElement<E>,
}

impl<E: RescueEngine> std::fmt::Debug for AccountContent<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AllocatedOperationBranch")
            .field("nonce", &self.nonce.get_number().get_value())
            .field("pub_key_hash", &self.pub_key_hash.get_number().get_value())
            .field("address", &self.address.get_number().get_value())
            .finish()
    }
}

impl<E: RescueEngine> AccountContent<E> {
    pub fn from_witness<CS: ConstraintSystem<E>>(
        mut cs: CS,
        witness: &AccountWitness<E>,
    ) -> Result<Self, SynthesisError> {
        let nonce = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "nonce"),
            || witness.nonce.grab(),
            zklink_crypto::params::NONCE_BIT_WIDTH,
        )?;

        let pub_key_hash = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "pub_key_hash"),
            || witness.pub_key_hash.grab(),
            zklink_crypto::params::NEW_PUBKEY_HASH_WIDTH,
        )?;

        let address = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "address"),
            || witness.address.grab(),
            zklink_crypto::params::ETH_ADDRESS_BIT_WIDTH,
        )?;

        Ok(Self {
            nonce,
            pub_key_hash,
            address,
        })
    }
}
