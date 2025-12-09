use std::fmt;

use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError, body::BoxBody};
use serde::Serialize;
use serde_json::{Value, json};
use utoipa::{IntoResponses, ToSchema};

use super::error::Error;

macro_rules! create {
    ($name:ident, $status:tt, $desc:tt) => {
        #[derive(Clone, Serialize, ToSchema, IntoResponses)]
        #[response(status = $status, description = $desc)]
        pub struct $name {
            #[schema(example = $desc)]
            pub message: String,
        }

        impl $name {
            pub fn new<T: ToString>(message: T) -> Self {
                Self {
                    message: message.to_string(),
                }
            }

            pub fn json(&self) -> Value {
                json!(self)
            }

            pub fn error(&self) -> Error {
                self.clone().into()
            }
        }

        impl Into<Error> for $name {
            fn into(self) -> Error {
                Error::$name {
                    message: self.message,
                }
            }
        }

        impl Into<Error> for &$name {
            fn into(self) -> Error {
                Error::$name {
                    message: self.message.clone(),
                }
            }
        }

        impl ResponseError for $name {
            fn error_response(&self) -> HttpResponse<BoxBody> {
                let error: Error = self.into();

                error.response()
            }
        }

        impl Responder for $name {
            type Body = BoxBody;

            fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
                self.error_response()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("message", &self.message)
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.json())
            }
        }
    };
}

create!(BadRequest, 400, "Bad Request");
create!(Unauthorized, 401, "Unauthorized");
create!(Forbidden, 403, "Forbidden");
create!(NotFound, 404, "Not Found");
create!(InternalServerError, 500, "Internal Server Error");
