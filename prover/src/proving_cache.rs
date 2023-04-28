use crate::SETUP_MIN_POW2;
use recover_state_config::RecoverStateConfig;
use zklink_circuit::exit_circuit::ZkLinkExitCircuit;
use zklink_crypto::bellman::bn256::Bn256;
use zklink_crypto::bellman::kate_commitment::{Crs, CrsForMonomialForm};
use zklink_crypto::bellman::plonk;
use zklink_crypto::bellman::plonk::better_cs::adaptor::TranspilationVariant;
use zklink_crypto::bellman::plonk::better_cs::cs::PlonkCsWidth4WithNextStepParams;
use zklink_crypto::bellman::plonk::SetupPolynomials;

pub struct ProvingCache {
    pub(crate) hints: Vec<(usize, TranspilationVariant)>,
    pub(crate) setup: SetupPolynomials<Bn256, PlonkCsWidth4WithNextStepParams>,
    pub(crate) key_monomial_form: Crs<Bn256, CrsForMonomialForm>,
}

impl ProvingCache {
    pub fn from_config(config: &RecoverStateConfig) -> anyhow::Result<Self> {
        let exit_circuit = ZkLinkExitCircuit::generate(config.layer1.get_max_chain_num());
        let hints = plonk::transpile(exit_circuit.clone())?;
        let setup = plonk::setup(exit_circuit, &hints)?;
        let size_log2 = setup.n.next_power_of_two().trailing_zeros();

        let size_log2 = std::cmp::max(size_log2, SETUP_MIN_POW2); // for exit circuit
        let key_monomial_form = crate::utils::get_universal_setup_monomial_form(
            &config.runtime.zklink_home,
            size_log2,
        )?;

        Ok(Self {
            hints,
            setup,
            key_monomial_form,
        })
    }
}
