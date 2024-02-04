use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput};

pub(crate) struct PaginationResponse {
    item: Ident,
}

impl PaginationResponse {
    pub(crate) fn new(input: DeriveInput) -> Self {
        Self { item: input.ident }
    }

    pub(crate) fn expand(&self) -> TokenStream {
        let item = &self.item;
        let name = Ident::new(&format!("{item}PaginationResponse"), Span::call_site());

        quote!(
            #[derive(
                Clone,
                ::serde::Deserialize,
                ::serde::Serialize,
                ::utoipa::ToSchema,
                ::utoipa::IntoResponses)
            ]
            #[response(status = 200, description = "OK")]
            #[serde(rename_all = "camelCase")]
            pub struct #name {
                #[schema(example = 451)]
                pub total: u64,
                #[schema(example = 1)]
                pub page: u64,
                #[schema(example = 10)]
                pub pages: u64,
                #[schema()]
                pub data: Vec<#item>,
            }

            impl ::actix_web::Responder for #name {
                type Body = ::actix_web::body::BoxBody;

                fn respond_to(self, _: &::actix_web::HttpRequest) -> ::actix_web::HttpResponse<Self::Body> {
                    ::actix_web::HttpResponse::Ok().json(self)
                }
            }

            impl ::actix_web::Responder for #item {
                type Body = ::actix_web::body::BoxBody;

                fn respond_to(self, _: &::actix_web::HttpRequest) -> ::actix_web::HttpResponse<Self::Body> {
                    ::actix_web::HttpResponse::Ok().json(self)
                }
            }

            //
        )
    }
}

pub(crate) struct PaginationRequest {
    item: Ident,
    data: Data,
}

impl PaginationRequest {
    pub(crate) fn new(input: DeriveInput) -> Self {
        Self {
            item: input.ident,
            data: input.data,
        }
    }

    pub(crate) fn expand(&self) -> TokenStream {
        let item = &self.item;
        let name = Ident::new(&format!("{item}PaginationRequest"), Span::call_site());
        let sort = Ident::new(&format!("{item}PaginationSort"), Span::call_site());

        let order = Ident::new(&format!("{item}PaginationOrder"), Span::call_site());
        let mut orderables = vec![Ident::new("CreatedAt", Span::call_site())];
        let mut default = None;

        if let Data::Struct(data) = &self.data {
            for field in &data.fields {
                // parse #[order]
                let orderable = field.attrs.iter().any(|attr| attr.path().is_ident("order"));
                // parse #[order(default)]
                let is_default = field.attrs.iter().any(|attr| {
                    if !attr.path().is_ident("order") {
                        return false;
                    }

                    attr.meta.path().is_ident("default")
                });

                if orderable {
                    let name = field.ident.as_ref().unwrap();
                    let name = capitalize(name);

                    if name.eq_ignore_ascii_case("CreatedAt") {
                        continue;
                    }

                    let name = Ident::new(&name, Span::call_site());

                    orderables.push(name.clone());

                    if is_default {
                        default = Some(name);
                    }
                }
            }
        }

        let default = default.unwrap_or_else(|| orderables[0].clone());

        quote!(
            #[derive(
                Clone,
                ::serde::Deserialize,
                ::serde::Serialize,
                ::utoipa::ToSchema
            )]
            #[serde(rename_all = "camelCase")]
            pub enum #order {
                #(#orderables,)*
            }

            impl Default for #order {
                fn default() -> Self {
                    Self::#default
                }
            }

            #[derive(
                Clone,
                Copy,
                ::serde::Deserialize,
                ::serde::Serialize,
                ::utoipa::ToSchema
            )]
            #[serde(rename_all = "camelCase")]
            pub enum #sort {
                Asc,
                Desc,
            }

            #[derive(
                Clone,
                ::serde::Deserialize,
                ::serde::Serialize,
                ::utoipa::ToSchema,
                ::utoipa::IntoParams,
            )]
            #[serde(rename_all = "camelCase")]
            #[into_params(parameter_in = Query)]
            pub struct #name {
                #[schema(example = 1, required = false)]
                page: Option<u64>,
                #[schema(example = 10)]
                limit: Option<u64>,
                #[schema()]
                search: Option<String>,
                #[schema()]
                sort: Option<#sort>,
                #[schema()]
                order: Option<#order>,
            }

            impl #name {
                pub fn page(&self) -> u64 {
                    self.page.unwrap_or(1)
                }

                pub fn limit(&self) -> u64 {
                    let limit = self.limit.unwrap_or(10);

                    if limit > 1000 {
                        1000
                    } else {
                        limit
                    }
                }

                pub fn offset(&self) -> u64 {
                    (self.page() - 1) * self.limit()
                }

                pub fn search(&self) -> Option<String> {
                    self.search.clone()
                }

                pub fn sort(&self) -> ::lighter_common::prelude::sea_orm::Order {
                    match self.sort.unwrap_or(#sort::Desc) {
                        #sort::Asc => ::lighter_common::prelude::sea_orm::Order::Asc,
                        #sort::Desc => ::lighter_common::prelude::sea_orm::Order::Desc,
                    }
                }

                pub fn order(&self) -> #order {
                    self.order.clone().unwrap_or_default()
                }
            }
        )
    }
}

fn capitalize<T: ToString>(input: T) -> String {
    let input = input.to_string();
    let words = input.split('_').collect::<Vec<_>>();

    words
        .iter()
        .map(|word| {
            let mut chars = word.chars();

            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello_world"), "HelloWorld");
        assert_eq!(capitalize("hello_world_"), "HelloWorld");
        assert_eq!(capitalize("_hello_world"), "HelloWorld");
        assert_eq!(capitalize("_hello_world_"), "HelloWorld");
        assert_eq!(
            capitalize("hello_world_hello_world"),
            "HelloWorldHelloWorld"
        );
        assert_eq!(
            capitalize("hello_world_hello_world_"),
            "HelloWorldHelloWorld"
        );
        assert_eq!(
            capitalize("_hello_world_hello_world"),
            "HelloWorldHelloWorld"
        );
        assert_eq!(
            capitalize("_hello_world_hello_world_"),
            "HelloWorldHelloWorld"
        );
    }
}
