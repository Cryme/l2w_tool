use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

struct FieldInfo {
    ident: Ident,
    ty: Type,
}

struct StructInfo {
    fields: Vec<FieldInfo>,
}

#[proc_macro_derive(ReadUnreal)]
pub fn read_unreal_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident.clone();

    let struct_info = match input.data {
        Data::Struct(data) => parse_struct(data),
        _ => panic!("ReadUnreal can only be derived for structs"),
    };

    let field_conversions = generate_field_conversions(&struct_info);

    let expanded = quote! {
        impl ReadUnreal for #struct_name {
            fn read_unreal<T: std::io::Read>(reader: &mut T) -> Self {
                Self {
                    #(#field_conversions)*
                }
            }
        }
    };

    expanded.into()
}

fn parse_struct(data: syn::DataStruct) -> StructInfo {
    let fields = match data.fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("ReadUnreal can only be derived for structs with named fields"),
    };

    let mut fields_inf = Vec::new();

    for field in fields {
        let ident = field.ident.expect("Named field must have an identifier");
        let ty = field.ty;

        fields_inf.push(FieldInfo { ident, ty });
    }

    StructInfo { fields: fields_inf }
}

fn generate_field_conversions(struct_info: &StructInfo) -> Vec<TokenStream> {
    struct_info
        .fields
        .iter()
        .map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote_spanned! {
                ident.span() => #ident: reader.read_unreal_value::<#ty>(),
            }
        })
        .collect()
}
