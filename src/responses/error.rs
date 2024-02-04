use std::collections::HashMap;
use std::fmt;

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError};
use sea_orm::{DbErr, TransactionError};
use serde_json::{json, Value};
use utoipa::openapi::{ObjectBuilder, RefOr, Schema, SchemaType};
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Eq)]
pub enum Error {
    // 400
    BadRequest {
        message: String,
    },
    // 401
    Unauthorized {
        message: String,
    },
    // 403
    Forbidden {
        message: String,
    },
    // 404
    NotFound {
        message: String,
    },
    // 422
    UnprocessableEntity {
        errors: HashMap<String, Vec<String>>,
    },
    // 500
    InternalServerError {
        message: String,
    },
}

impl Error {
    pub fn json(&self) -> Value {
        if let Self::UnprocessableEntity { errors } = self {
            return json!({
                "errors": errors
            });
        }

        let message = match self {
            Self::BadRequest { message } => message,
            Self::Unauthorized { message } => message,
            Self::Forbidden { message } => message,
            Self::NotFound { message } => message,
            Self::InternalServerError { message } => message,
            _ => "Unknown error",
        };

        json!({
            "message": message,
        })
    }

    pub fn status_code(&self) -> StatusCode {
        use Error::*;

        match self {
            BadRequest { message: _ } => StatusCode::BAD_REQUEST,
            Unauthorized { message: _ } => StatusCode::UNAUTHORIZED,
            Forbidden { message: _ } => StatusCode::FORBIDDEN,
            NotFound { message: _ } => StatusCode::NOT_FOUND,
            UnprocessableEntity { errors: _ } => StatusCode::UNPROCESSABLE_ENTITY,
            InternalServerError { message: _ } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn response(&self) -> HttpResponse {
        use Error::*;

        let mut response = match self {
            BadRequest { message: _ } => HttpResponse::BadRequest(),
            Unauthorized { message: _ } => HttpResponse::Unauthorized(),
            Forbidden { message: _ } => HttpResponse::Forbidden(),
            NotFound { message: _ } => HttpResponse::NotFound(),
            UnprocessableEntity { errors: _ } => HttpResponse::UnprocessableEntity(),
            InternalServerError { message: _ } => HttpResponse::InternalServerError(),
        };

        response.json(self.json())
    }
}

impl From<DbErr> for Error {
    fn from(value: DbErr) -> Self {
        use DbErr::*;

        match value {
            RecordNotFound(_) => Self::NotFound {
                message: "Not found".to_string(),
            },
            _ => Self::InternalServerError {
                message: value.to_string(),
            },
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::BadRequest {
            message: value.to_string(),
        }
    }
}

impl From<TransactionError<DbErr>> for Error {
    fn from(value: TransactionError<DbErr>) -> Self {
        Self::InternalServerError {
            message: value.to_string(),
        }
    }
}

impl Responder for Error {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        self.response()
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        self.response()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.json())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.json())
    }
}

impl ToSchema<'_> for Error {
    fn schema() -> (&'static str, RefOr<Schema>) {
        let schema = ObjectBuilder::new()
            .schema_type(SchemaType::Object)
            .property(
                "message",
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .example(Some(json!("Not found")))
                    .build(),
            )
            .build();

        ("Error", schema.into())
    }
}
