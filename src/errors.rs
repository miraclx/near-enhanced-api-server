use paperclip::actix::{api_v2_errors, Apiv2Schema};

use near_jsonrpc_client::errors::JsonRpcError;
use near_jsonrpc_primitives::types::query::RpcQueryError;

#[derive(Debug, strum::EnumIter)]
pub enum ErrorKind {
    DBError(String),
    InvalidInput(String),
    InternalError(String),
    NotImplemented(String),
    RPCError(String),
}

pub(crate) fn contract_not_found(
    contract_id: &near_primitives::types::AccountId,
    block_height: u64,
) -> ErrorKind {
    ErrorKind::InvalidInput(format!(
        "The account `{}` does not implement the desired contract at block {}",
        contract_id, block_height
    ))
}

/// Instead of utilizing HTTP status codes to describe node errors (which often
/// do not have a good analog), rich errors are returned using this object.
#[api_v2_errors(
    code = 500,
    description = "See the inner `code` value to get more details"
)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct Error {
    /// Code is a network-specific error code. If desired, this code can be
    /// equivalent to an HTTP status code.
    pub code: u32,

    /// Message is a network-specific error message.
    pub message: String,

    /// An error is retriable if the same request may succeed if submitted
    /// again.
    pub retriable: bool,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let retriable = if self.retriable { " (retriable)" } else { "" };
        write!(f, "Error #{}{}: {}", self.code, retriable, self.message)
    }
}

impl Error {
    pub(crate) fn from_error_kind(err: ErrorKind) -> Self {
        match err {
            ErrorKind::DBError(message) => Self {
                code: 500,
                message: format!("DB Error: {}", message),
                retriable: true,
            },
            ErrorKind::InvalidInput(message) => Self {
                code: 400,
                message: format!("Invalid Input: {}", message),
                retriable: false,
            },
            ErrorKind::InternalError(message) => Self {
                code: 500,
                message: format!("Internal Error: {}", message),
                retriable: true,
            },
            ErrorKind::NotImplemented(message) => Self {
                code: 500,
                message: format!(
                    "Sorry! Please wait a bit, we are working on that: {}",
                    message
                ),
                retriable: true,
            },
            ErrorKind::RPCError(message) => Self {
                code: 500,
                message: format!("RPC error: {}", message),
                retriable: true,
            },
        }
    }
}

impl<T> From<T> for Error
where
    T: Into<ErrorKind>,
{
    fn from(err: T) -> Self {
        Self::from_error_kind(err.into())
    }
}

impl actix_web::ResponseError for Error {
    fn error_response(&self) -> actix_web::HttpResponse {
        let data = paperclip::actix::web::Json(self);
        actix_web::HttpResponse::InternalServerError().json(data)
    }
}

impl From<JsonRpcError<RpcQueryError>> for ErrorKind {
    fn from(error: JsonRpcError<RpcQueryError>) -> Self {
        Self::RPCError(format!("{:#?}", error))
    }
}

impl From<serde_json::Error> for ErrorKind {
    fn from(error: serde_json::Error) -> Self {
        Self::InternalError(format!("Serialization failure: {:#?}", error))
    }
}

impl From<near_primitives::account::id::ParseAccountError> for ErrorKind {
    fn from(error: near_primitives::account::id::ParseAccountError) -> Self {
        Self::InternalError(format!("Could not parse account: {:#?}", error))
    }
}
