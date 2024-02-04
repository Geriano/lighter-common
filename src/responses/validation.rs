use std::{collections::HashMap, fmt};

use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder, ResponseError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{
    openapi::{ObjectBuilder, RefOr, Schema, SchemaType},
    IntoResponses, ToSchema,
};

use super::error::Error;

#[derive(Clone, Debug, Deserialize, Serialize, IntoResponses)]
#[response(status = 422, description = "Unprocessable Entity")]
pub struct Validation {
    pub errors: HashMap<String, Vec<String>>,
}

impl Validation {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add<F: ToString, M: ToString>(&mut self, field: F, message: M) {
        self.errors
            .entry(field.to_string())
            .or_insert_with(Vec::new)
            .push(message.to_string());
    }

    pub fn get<T: ToString>(&self, field: T) -> Vec<String> {
        self.errors
            .get(&field.to_string())
            .cloned()
            .unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_error<T: ToString>(&self, field: T) -> bool {
        !self.get(field).is_empty()
    }
}

impl From<Error> for Validation {
    fn from(value: Error) -> Self {
        if let Error::UnprocessableEntity { errors } = value {
            return Self { errors };
        }

        return Self::new();
    }
}

impl Into<Error> for &Validation {
    fn into(self) -> Error {
        Error::UnprocessableEntity {
            errors: self.errors.clone(),
        }
    }
}

impl Into<Error> for Validation {
    fn into(self) -> Error {
        Error::UnprocessableEntity {
            errors: self.errors,
        }
    }
}

impl ResponseError for Validation {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let error: Error = self.into();

        error.response()
    }
}

impl Responder for Validation {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        self.error_response()
    }
}

impl fmt::Display for Validation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", json!(self))
    }
}

impl ToSchema<'_> for Validation {
    fn schema() -> (&'static str, RefOr<Schema>) {
        let errors = ObjectBuilder::new()
            .schema_type(SchemaType::Array)
            .to_array_builder()
            .items(
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .example(Some(json!("Password must be at least 8 characters long")))
                    .build(),
            )
            .items(
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .example(Some(json!(
                        "Password must contain at least one uppercase letter"
                    )))
                    .build(),
            )
            .build();

        let schema = ObjectBuilder::new()
            .schema_type(SchemaType::Object)
            .property(
                "errors",
                ObjectBuilder::new()
                    .schema_type(SchemaType::Object)
                    .property("password", errors)
                    .build(),
            )
            .build();

        ("Validation", schema.into())
    }
}
