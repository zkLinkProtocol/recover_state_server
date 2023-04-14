use crate::exit_circuit::*;

#[derive(Clone, Debug)]
pub struct AccountWitness<E: RescueEngine> {
    pub nonce: Option<E::Fr>,
    pub pub_key_hash: Option<E::Fr>,
    pub address: Option<E::Fr>,
}

impl<E: RescueEngine> Default for AccountWitness<E> {
    fn default() -> Self {
        Self {
            nonce: None,
            pub_key_hash: None,
            address: None,
        }
    }
}

impl<E: RescueEngine> AccountWitness<E> {
    pub fn circuit_init() -> Self {
        Self {
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
