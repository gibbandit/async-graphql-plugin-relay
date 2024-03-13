use async_graphql::{ComplexObject, Error, SimpleObject};
use async_graphql_plugin_relay::{RelayContext, RelayNode, RelayNodeID, RelayNodeObject};
use async_trait::async_trait;

use crate::Node;

#[derive(Debug, SimpleObject, RelayNodeObject)]
#[graphql(complex)]
pub struct User {
    pub id: RelayNodeID<Self>,
    pub name: String,
    pub role: String,
}

#[async_trait]
impl RelayNode for User {
    type TNode = Node;

    async fn get(ctx: RelayContext, id: RelayNodeID<Self>) -> Result<Option<Self::TNode>, Error> {
        let ctx_str = ctx.get::<String>().unwrap();
        println!("Getting User: {:?} with context {}", id, ctx_str);

        Ok(Some(
            User {
                id: RelayNodeID::new("92ba0c2d-4b4e-4e29-91dd-8f96a078c3ff"),
                name: "Oscar".to_string(),
                role: "Testing123".to_string(),
            }
            .into(),
        ))
    }
}

#[ComplexObject]
impl User {
    pub async fn test(&self) -> String {
        "testing".to_string()
    }
}
