use utoipa::openapi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::responses;

#[derive(OpenApi)]
#[openapi(components(schemas(
    responses::Error,
    responses::Validation,
    responses::BadRequest,
    responses::Unauthorized,
    responses::Forbidden,
    responses::NotFound,
    responses::InternalServerError,
)))]
pub struct Builtin;

impl Modify for Builtin {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        openapi.merge(Self::openapi());
    }
}

pub struct Authentication;

impl Modify for Authentication {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
