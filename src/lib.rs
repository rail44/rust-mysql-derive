#![feature(proc_macro, proc_macro_lib)]
#![cfg(not(test))]

extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;
#[macro_use]
extern crate quote;

#[proc_macro_derive(FromMysqlRow)]
pub fn derive_from_mysql_row(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_macro_input(&source).unwrap();
    let expanded = expand_from_mysql_row(&ast);
    expanded.parse().unwrap()
}

fn expand_build_struct_field(field_name: syn::Ident, row: syn::Expr) -> quote::Tokens {
    let field_name_str = format!("{}", field_name);
    quote! {
        #field_name : #row . take(#field_name_str).ok_or_else(|| _mysql::Error::FromRowError(row.clone()))?, 
    }
}

fn expand_from_mysql_row(ast: &syn::MacroInput) -> quote::Tokens {
    let fields = match &ast.body {
        &syn::Body::Struct(
            syn::VariantData::Struct(ref items)
        ) => items,
        _ => panic!("#[derive(FromMysqlRow)] can only be used with structs"),
    };

    let name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let dummy_const = syn::Ident::new(format!("_DUMMY_CONST_MYSQL_DERIVE_{}", ast.ident));
    let row_arg_expr = syn::parse_expr("row").unwrap();

    let mut struct_fields = quote::Tokens::new();
    for field in fields {
        let ident = field.ident.clone();
        struct_fields.append(
            expand_build_struct_field(ident.unwrap(), row_arg_expr.clone()).as_str()
        );
    }

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate mysql as _mysql;


            #[automatically_derived]
            impl #impl_generics _mysql::FromRow for #name #ty_generics #where_clause {
                fn from_row(row: _mysql::Row) -> #name #ty_generics #where_clause {
                    Self::from_row_opt(row).unwrap()
                }

                fn from_row_opt(mut #row_arg_expr: _mysql::Row) -> _mysql::Result<#name #ty_generics #where_clause> {
                    Ok(#name {
                        #struct_fields
                    })
                }
            }
        };
    }
}
