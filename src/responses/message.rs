use actix_web::body::BoxBody;
use actix_web::{HttpRequest, HttpResponse, Responder};
use utoipa::IntoResponses;

#[derive(IntoResponses)]
#[response(status = 200, description = "Ok")]
pub struct Success;

impl Responder for Success {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok().finish()
    }
}
