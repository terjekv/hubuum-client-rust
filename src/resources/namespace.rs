use api_resource_derive::ApiResource;

#[allow(dead_code)]
#[derive(ApiResource)]
pub struct NamespaceResource {
    #[api(read_only)]
    pub id: i32,
    #[api(table_rename = "Name")]
    pub name: String,
    #[api(table_rename = "Description")]
    pub description: String,
    #[api(post_only, table_rename = "Group")]
    pub group_id: i32, // This is the group that the namespace belongs to and is set on creation.
    #[api(read_only, table_rename = "Created")]
    pub created_at: chrono::NaiveDateTime,
    #[api(read_only, table_rename = "Updated")]
    pub updated_at: chrono::NaiveDateTime,
}
