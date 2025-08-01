use std::borrow::Cow;

use api_resource_derive::ApiResource;

use crate::{
    client::sync::{EmptyPostParams, Handle},
    endpoints::Endpoint,
    ApiError, User,
};

#[allow(dead_code)]
#[derive(ApiResource)]
pub struct GroupResource {
    #[api(read_only)]
    pub id: i32,
    #[api(table_rename = "Name")]
    pub groupname: String,
    #[api(table_rename = "Description")]
    pub description: String,
    #[api(read_only, table_rename = "Created")]
    pub created_at: chrono::NaiveDateTime,
    #[api(read_only, table_rename = "Updated")]
    pub updated_at: chrono::NaiveDateTime,
}

impl Handle<Group> {
    pub fn add_user(&self, user_id: i32) -> Result<(), ApiError> {
        let url_params = vec![
            (
                Cow::Borrowed("group_id"),
                self.resource().id.to_string().into(),
            ),
            (Cow::Borrowed("user_id"), user_id.to_string().into()),
        ];

        self.client().request_with_endpoint::<EmptyPostParams, ()>(
            reqwest::Method::POST,
            &Endpoint::GroupMembersAddRemove,
            url_params,
            vec![],
            EmptyPostParams {},
        )?;
        Ok(())
    }

    pub fn remove_user(&self, user_id: i32) -> Result<(), ApiError> {
        let url_params = vec![
            (
                Cow::Borrowed("group_id"),
                self.resource().id.to_string().into(),
            ),
            (Cow::Borrowed("user_id"), user_id.to_string().into()),
        ];

        self.client().request_with_endpoint::<EmptyPostParams, ()>(
            reqwest::Method::DELETE,
            &Endpoint::GroupMembersAddRemove,
            url_params,
            vec![],
            EmptyPostParams {},
        )?;
        Ok(())
    }

    pub fn members(&self) -> Result<Vec<Handle<User>>, ApiError> {
        let url_params = vec![(
            Cow::Borrowed("group_id"),
            self.resource().id.to_string().into(),
        )];
        let res = self
            .client()
            .request_with_endpoint::<EmptyPostParams, Vec<User>>(
                reqwest::Method::GET,
                &Endpoint::GroupMembers,
                url_params,
                vec![],
                EmptyPostParams {},
            )?;

        match res {
            None => Ok(vec![]),
            Some(users) => Ok(users
                .into_iter()
                .map(|user| Handle::new(self.client().clone(), user))
                .collect()),
        }
    }
}
