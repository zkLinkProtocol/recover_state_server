#![allow(dead_code)]
use serde::Serialize;
use tracing::error;

#[derive(Debug, Serialize, Clone)]
pub struct ExodusResponse<T: Serialize + Clone>{
    pub code: i32,
    pub data: Option<T>,
    pub err_msg: Option<String>,
}

impl<T: Serialize + Clone> From<ExodusError> for ExodusResponse<T> {
    fn from(code: ExodusError) -> Self {
        Self{
            code: code as i32,
            data: None,
            err_msg: Some(code.to_string()),
        }
    }
}

impl<T: Serialize + Clone> ExodusResponse<T> {
    #[allow(non_snake_case)]
    pub fn Ok() -> ExodusResponse<T>{
        Self{
            code: ExodusError::Ok as i32,
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
pub enum ExodusError {
    Ok = 0,
    ProofNotBegin = 50,
    ProofGenerating = 51,
    ProofCompleted = 52,
    NonBalance = 60,

    TokenNotExist = 101,
    AccountNotExist = 102,
    ChainNotExist = 103,
    ExitProofTaskNotExist = 104,

    InvalidL1L2Token = 201,

    InternalErr=500
}

impl From<anyhow::Error> for ExodusError {
    fn from(err: anyhow::Error) -> Self {
        error!("Exodus server internal error: {}", err);
        ExodusError::InternalErr
    }
}

impl ToString for ExodusError {
    fn to_string(&self) -> String {
        match self {
            // Normal response
            ExodusError::Ok => "Ok",
            ExodusError::ProofNotBegin => "The proof task has not yet begun",
            ExodusError::ProofGenerating => "The proof task is running",
            ExodusError::ProofCompleted => "The task has been completed",
            ExodusError::NonBalance => "The token of the account is no balance",

            // Not exist info
            ExodusError::TokenNotExist => "The token not exist",
            ExodusError::AccountNotExist => "The account not exist",
            ExodusError::ChainNotExist => "The chain not exist",
            ExodusError::ExitProofTaskNotExist => "The exit proof task not exist",

            // Invalid parameters
            ExodusError::InvalidL1L2Token => "The relationship between l1 token and l2 token is incorrect",

            // Internal error,
            ExodusError::InternalErr => "Exodus server internal error",
        }.to_string()
    }
}