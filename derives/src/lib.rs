use proc_macro::TokenStream;
use syn::parse_macro_input;

mod pagination;

#[proc_macro_derive(PaginationResponse)]
pub fn pagination_response_derive(input: TokenStream) -> TokenStream {
    pagination::PaginationResponse::new(parse_macro_input!(input))
        .expand()
        .into()
}

#[proc_macro_derive(PaginationRequest, attributes(order))]
pub fn pagination_request_derive(input: TokenStream) -> TokenStream {
    pagination::PaginationRequest::new(parse_macro_input!(input))
        .expand()
        .into()
}
