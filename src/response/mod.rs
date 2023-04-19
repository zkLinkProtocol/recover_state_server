#![allow(dead_code)]
mod types;

use serde::{Deserialize, Serialize};
use tracing::error;
pub use types::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExodusResponse<T: Serialize + Clone> {
    pub code: i32,
    pub data: Option<T>,
    pub err_msg: Option<String>,
}

impl<T: Serialize + Clone> From<ExodusStatus> for ExodusResponse<T> {
    fn from(code: ExodusStatus) -> Self {
        Self {
            code: code as i32,
            data: None,
            err_msg: Some(code.to_string()),
        }
    }
}

impl<T: Serialize + Clone> ExodusResponse<T> {
    #[allow(non_snake_case)]
    pub fn Ok() -> ExodusResponse<T> {
        Self {
            code: ExodusStatus::Ok as i32,
            data: None,
            err_msg: None,
        }
    }

    pub fn data(mut self, data: T) -> ExodusResponse<T> {
        self.data = data.into();
        self
    }
}

#[derive(Copy, Clone, Serialize)]
pub enum ExodusStatus {
    Ok = 0,
    ProofTaskAlreadyExists = 50,
    ProofGenerating = 51,
    ProofCompleted = 52,
    NonBalance = 60,
    RecoverStateUnfinished = 70,

    TokenNotExist = 101,
    AccountNotExist = 102,
    ChainNotExist = 103,
    ExitProofTaskNotExist = 104,

    InvalidL1L2Token = 201,
    ProofsLoadTooMany = 202,

    InternalErr = 500,
}

impl From<anyhow::Error> for ExodusStatus {
    fn from(err: anyhow::Error) -> Self {
        error!("Exodus server internal error: {}", err);
        ExodusStatus::InternalErr
    }
}

impl ToString for ExodusStatus {
    fn to_string(&self) -> String {
        match self {
            // Normal response
            ExodusStatus::Ok => "Ok",
            ExodusStatus::ProofTaskAlreadyExists => "The proof Task already exists",
            ExodusStatus::ProofGenerating => "The proof task is running",
            ExodusStatus::ProofCompleted => "The task has been completed",
            ExodusStatus::NonBalance => "The token of the account is no balance",
            ExodusStatus::RecoverStateUnfinished => "Recovering state is unfinished",

            // Not exist info
            ExodusStatus::TokenNotExist => "The token not exist",
            ExodusStatus::AccountNotExist => "The account not exist",
            ExodusStatus::ChainNotExist => "The chain not exist",
            ExodusStatus::ExitProofTaskNotExist => "The exit proof task not exist",

            // Invalid parameters
            ExodusStatus::InvalidL1L2Token => {
                "The relationship between l1 token and l2 token is incorrect"
            }
            ExodusStatus::ProofsLoadTooMany => "There are too many proofs to obtain",

            // Internal error,
            ExodusStatus::InternalErr => "Exodus server internal error",
        }
        .to_string()
    }
}
