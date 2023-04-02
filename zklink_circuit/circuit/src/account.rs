use crate::circuit::*;

#[derive(Clone, Debug)]
pub struct AccountWitness<E: RescueEngine> {
    pub nonce: Option<E::Fr>,
    pub pub_key_hash: Option<E::Fr>,
    pub address: Option<E::Fr>,
}

impl<E:RescueEngine> Default for AccountWitness<E>
{
    fn default() -> Self {
        Self{
            nonce:None,
            pub_key_hash: None,
            address: None,
        }
    }
}

impl<E: RescueEngine> AccountWitness<E> {
    pub fn circuit_init() -> Self {
        Self{
            nonce: Some(E::Fr::zero()),
            pub_key_hash: Some(E::Fr::zero()),
            address: Some(E::Fr::zero()),
        }
    }

    pub fn from_circuit_account(circuit_account: &account::CircuitAccount<E>) -> Self {
        Self {
            nonce: Some(circuit_account.nonce),
            pub_key_hash: Some(circuit_account.pub_key_hash),
            address: Some(circuit_account.address),
        }
    }
}

pub struct AccountContent<E: RescueEngine> {
    pub nonce: CircuitElement<E>,
    pub pub_key_hash: CircuitElement<E>,
    pub address: CircuitElement<E>,
}

impl<E:RescueEngine> std::fmt::Debug for AccountContent<E>{
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
            || Ok(witness.nonce.grab()?),
            zklink_crypto::params::NONCE_BIT_WIDTH,
        )?;

        let pub_key_hash = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "pub_key_hash"),
            || Ok(witness.pub_key_hash.grab()?),
            zklink_crypto::params::NEW_PUBKEY_HASH_WIDTH,
        )?;

        let address = CircuitElement::from_fe_with_known_length(
            cs.namespace(|| "address"),
            || Ok(witness.address.grab()?),
            zklink_crypto::params::ETH_ADDRESS_BIT_WIDTH,
        )?;

        Ok(Self {
            nonce,
            pub_key_hash,
            address,
        })
    }
}
