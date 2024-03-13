use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput};

#[macro_use]
extern crate quote;
extern crate proc_macro;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(relay))]
struct RelayNodeObjectAttributes {
    node_typename: Option<String>,
}

/// The RelayNodeObject macro is applied to a type to automatically implement the RelayNodeStruct trait.
/// ```
/// #[derive(SimpleObject, RelayNodeObject)] // See the 'RelayNodeObject' derive macro
/// #[graphql(complex)]
/// #[relay(node_typename = "User")] // This controls the 'RelayNodeObject' macro. In this case the prefix is 'User', the default is in the name of the struct.
/// pub struct User {
///     pub id: RelayNodeID<User>,
///     pub name: String,
///     pub role: String,
/// }
/// ```
#[proc_macro_derive(RelayNodeObject, attributes(relay))]
pub fn derive_relay_node_object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let attrs = RelayNodeObjectAttributes::from_derive_input(&input)
        .expect("Error parsing 'RelayNodeObject' macro options!");
    let DeriveInput { ident, data, .. } = input;

    if !matches!(data, Data::Struct(_)) {
        panic!("The 'RelayNodeObject' macro can only be used on structs!");
    }

    let value = if let Some(node_typename) = attrs.node_typename {
        node_typename
    } else {
        ident.to_string()
    };

    quote! {
        impl async_graphql_plugin_relay::RelayNodeStruct for #ident {
            const ID_TYPENAME: &'static str = #value;
        }
    }
    .into()
}

/// The RelayInterface macro is applied to a GraphQL Interface enum to allow it to be used for Relay's node query.
/// This enum should contain all types that that exist in your GraphQL schema to work as designed in the Relay server specification.
/// ```
/// #[derive(Interface, RelayInterface)] // See the 'RelayInterface' derive macro
/// #[graphql(field(name = "id", type = "ID"))]
/// pub enum Node {
///     User(User),
///     Tenant(Tenant),
///    // Put all of your Object's in this enum
/// }
/// ```
#[proc_macro_derive(RelayInterface)]
pub fn derive_relay_interface(input: TokenStream) -> TokenStream {
    let DeriveInput { data, .. } = parse_macro_input!(input);

    let node_matchers;
    if let Data::Enum(data) = &data {
        node_matchers = data.variants.iter().map(|variant| {
            let variant_ident = &variant.ident;
            quote! {
                <#variant_ident as async_graphql_plugin_relay::RelayNodeStruct>::ID_TYPENAME => {
                    <#variant_ident as async_graphql_plugin_relay::RelayNode>::get(
                        ctx,
                        async_graphql_plugin_relay::RelayNodeID::<#variant_ident>::new_from_relay_id(
                            relay_id.to_string(),
                        )?,
                    )
                    .await?
                    .ok_or_else(|| async_graphql::Error::new("A node with the specified id could not be found!"))
                }
            }
        });
    } else {
        panic!("The 'RelayNodeObject' macro can only be used on enums!");
    }

    quote! {
        #[async_graphql_plugin_relay::_async_trait]
        impl async_graphql_plugin_relay::RelayNodeInterface for Node {
            async fn fetch_node(ctx: async_graphql_plugin_relay::RelayContext, relay_id: String) -> Result<Self, async_graphql::Error> {
                use async_graphql_plugin_relay::_Engine as _;
                let decoded_id_vec = async_graphql_plugin_relay::_URL_SAFE
            .decode(relay_id.clone())
            .map_err(|_err| Error::new("invalid relay id provided to node query!"))?;
        let decoded_id = String::from_utf8(decoded_id_vec)
            .map_err(|_err| Error::new("invalid relay id provided to node query!"))?;

        let (typename, id) = match decoded_id.split_once(':') {
            Some((typename, id)) => (typename, id),
            None => {
                return Err(Error::new("Invalid relay id provided to node query!"));
            }
        };
                match typename {
                    #(#node_matchers)*
                    _ => Err(async_graphql::Error::new("A node with the specified id could not be found!")),
                }
            }
        }
            }
    .into()
}

// TODO: Unit tests
