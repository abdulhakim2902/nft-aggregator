use crate::schema::contracts;
use diesel::prelude::*;
use field_count::FieldCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Clone, Debug, Default, Deserialize, FieldCount, Identifiable, Insertable, Serialize, Queryable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = contracts)]
pub struct Contract {
    pub id: Option<Uuid>,
    pub key: Option<String>,
    pub type_: Option<String>,
    pub name: Option<String>,
}
