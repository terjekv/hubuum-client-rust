use api_resource_derive::ApiResource;

#[allow(dead_code)]
#[derive(ApiResource)]
pub struct NamespaceResource {
    #[api(read_only)]
    pub id: i32,
    pub name: String,
    pub description: String,
    #[api(post_only)]
    pub group_id: i32, // This is the group that the namespace belongs to and is set on creation.
    #[api(read_only)]
    pub created_at: chrono::NaiveDateTime,
    #[api(read_only)]
    pub updated_at: chrono::NaiveDateTime,
}
