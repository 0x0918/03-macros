use darling::{ast::Data, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Generics, Ident, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(deref))]
struct AutoDerefInfo {
    ident: Ident,
    generics: Generics,
    data: Data<(), AutoDerefFieldInfo>,
    #[darling(default)]
    mutable: bool,
    #[darling(default)]
    field: Option<Ident>,
}

#[derive(Debug, FromField)]
struct AutoDerefFieldInfo {
    ident: Option<Ident>,
    ty: Type,
}

pub fn process_auto_deref(input: DeriveInput) -> TokenStream {
    let AutoDerefInfo {
        ident,
        generics,
        data: Data::Struct(fields),
        mutable,
        field,
    } = AutoDerefInfo::from_derive_input(&input).unwrap()
    else {
        panic!("AutoDeref only works on structs");
    };

    let (fd, ty) = if let Some(field) = field {
        match fields.iter().find(|f| f.ident.as_ref().unwrap() == &field) {
            Some(f) => (field, &f.ty),
            None => panic!("field {:?} not found in the data structure", field),
        }
    } else {
        // if only 1 field, use that field
        if fields.len() == 1 {
            let f = fields.iter().next().unwrap();
            (f.ident.as_ref().unwrap().clone(), &f.ty)
        } else {
            panic!("AutoDeref only works on structs with 1 field or with field attribute");
        }
    };

    let mut code = vec![quote! {
        impl #generics std::ops::Deref for #ident #generics {
            type Target = #ty;

            fn deref(&self) -> &Self::Target {
                &self.#fd
            }
        }
    }];

    if mutable {
        code.push(quote! {
            impl #generics std::ops::DerefMut for #ident #generics {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.#fd
                }
            }
        })
    }

    quote! {#(#code)*}
}
