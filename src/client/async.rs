use log::trace;
use serde_urlencoded;
use std::marker::PhantomData;

use super::{Authenticated, ClientCore, IntoResourceFilter, Unauthenticated, UrlParams};
use crate::endpoints::Endpoint;
use crate::errors::ApiError;
use crate::resources::ApiResource;
use crate::resources::{Class, User};
use crate::types::{BaseUrl, Credentials, FilterOperator, Token};
use crate::QueryFilter;

#[derive(Debug, Clone)]
pub struct Client<S> {
    http_client: reqwest::Client,
    base_url: BaseUrl,
    state: S,
}

impl<S> ClientCore for Client<S> {
    fn build_url(&self, endpoint: &Endpoint, _url_params: UrlParams) -> String {
        format!(
            "{}{}",
            self.base_url.with_trailing_slash(),
            endpoint.trim_start_matches('/')
        )
    }
}

impl Client<Unauthenticated> {
    pub fn new(base_url: BaseUrl) -> Self {
        Client {
            http_client: reqwest::Client::new(),
            base_url,
            state: Unauthenticated,
        }
    }
}

impl Client<Unauthenticated> {
    pub async fn login(self, credentials: Credentials) -> Result<Client<Authenticated>, ApiError> {
        let token: Token = self
            .http_client
            .post(&self.build_url(&Endpoint::Login, UrlParams::default()))
            .json(&credentials)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(Client {
            http_client: self.http_client,
            base_url: self.base_url,
            state: Authenticated { token: token.token },
        })
    }

    pub async fn login_with_token(self, token: Token) -> Result<Client<Authenticated>, ApiError> {
        let status = self
            .http_client
            .get(self.build_url(&Endpoint::LoginWithToken, UrlParams::default()))
            .header("Authorization", format!("Bearer {}", token.token))
            .send()
            .await?;

        if status.status().is_success() {
            Ok(Client {
                http_client: self.http_client,
                base_url: self.base_url,
                state: Authenticated { token: token.token },
            })
        } else {
            Err(ApiError::InvalidToken)
        }
    }
}

impl Client<Authenticated> {
    pub fn get_token(&self) -> &str {
        &self.state.token
    }

    pub async fn get<R: ApiResource>(
        &self,
        resource: R,
        params: R::GetParams,
    ) -> Result<Vec<R::GetOutput>, ApiError> {
        let endpoint = resource.endpoint();
        let url = self.build_url(&endpoint, UrlParams::default());

        let query = serde_urlencoded::to_string(&params)?;
        let url = if !query.is_empty() {
            format!("{}?{}", url, query)
        } else {
            url
        };

        trace!("GET {}", url);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.state.token))
            .send()
            .await?
            .error_for_status()?;

        trace!("Response: {:?}", response);
        let obj: Vec<R::GetOutput> = response.json().await?;
        Ok(obj)
    }

    pub async fn search<R: ApiResource>(
        &self,
        resource: R,
        params: Vec<QueryFilter>,
    ) -> Result<Vec<R::GetOutput>, ApiError> {
        let endpoint = resource.endpoint();
        let params = serde_urlencoded::to_string(&params)?;

        let url = format!(
            "{}?{}",
            self.build_url(&endpoint, UrlParams::default()),
            params
        );

        trace!("GET {}", url);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.state.token))
            .send()
            .await?
            .error_for_status()?;

        trace!("Response: {:?}", response);
        let obj: Vec<R::GetOutput> = response.json().await?;
        Ok(obj)
    }

    pub async fn post<R: ApiResource>(
        &self,
        resource: R,
        params: R::PostParams,
    ) -> Result<R::PostOutput, ApiError> {
        let endpoint = resource.endpoint();
        let url = self.build_url(&endpoint, UrlParams::default());

        trace!("POST {} with {:?}", &url, params);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.state.token))
            .json(&params)
            .send()
            .await?
            .error_for_status()?;

        trace!("Response: {:?}", response);
        let obj: R::PostOutput = response.json().await?;
        Ok(obj)
    }

    pub async fn patch<R: ApiResource>(
        &self,
        resource: R,
        id: i32,
        params: R::PatchParams,
    ) -> Result<R::PatchOutput, ApiError> {
        let endpoint = resource.endpoint();
        let url = format!("{}/{}", self.build_url(&endpoint, UrlParams::default()), id);

        trace!("PATCH {} with {:?}", &url, params);

        let response = self
            .http_client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.state.token))
            .json(&params)
            .send()
            .await?
            .error_for_status()?;

        trace!("Response: {:?}", response);
        let obj: R::PatchOutput = response.json().await?;
        Ok(obj)
    }

    pub async fn delete<R: ApiResource>(
        &self,
        resource: R,
        id: i32,
    ) -> Result<R::DeleteOutput, ApiError> {
        let endpoint = resource.endpoint();
        let url = format!("{}/{}", self.build_url(&endpoint, UrlParams::default()), id);

        trace!("DELETE {}", &url);

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.state.token))
            .send()
            .await?
            .error_for_status()?;

        trace!("Response: {:?}", response);
        let obj: R::DeleteOutput = response.json().await?;
        Ok(obj)
    }

    pub fn users(&self) -> Resource<User> {
        Resource::new(self.clone())
    }

    pub fn classes(&self) -> Resource<Class> {
        Resource::new(self.clone())
    }
}

pub struct FilterBuilder<T: ApiResource> {
    client: Client<Authenticated>,
    filters: Vec<(String, FilterOperator, String)>,
    _phantom: PhantomData<T>,
}

impl<T: ApiResource> FilterBuilder<T> {
    fn new(client: Client<Authenticated>) -> Self {
        FilterBuilder {
            client,
            filters: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn add_filter<V: ToString>(mut self, field: &str, op: FilterOperator, value: V) -> Self {
        self.filters
            .push((field.to_string(), op, value.to_string()));
        self
    }

    pub async fn execute(self) -> Result<Vec<T::GetOutput>, ApiError> {
        let params = T::build_params(self.filters);
        self.client.search::<T>(T::default(), params).await
    }
}

pub struct Resource<T: ApiResource> {
    client: Client<Authenticated>,
    _phantom: PhantomData<T>,
}

impl<T: ApiResource> Resource<T> {
    fn new(client: Client<Authenticated>) -> Self {
        Resource {
            client,
            _phantom: PhantomData,
        }
    }

    pub fn find(&self) -> FilterBuilder<T> {
        FilterBuilder::new(self.client.clone())
    }

    pub async fn filter<F: IntoResourceFilter<T>>(
        &self,
        filter: F,
    ) -> Result<Vec<T::GetOutput>, ApiError> {
        let params = filter.into_resource_filter();
        self.client.search::<T>(T::default(), params).await
    }

    pub async fn create(&self, params: T::PostParams) -> Result<T::PostOutput, ApiError> {
        self.client.post::<T>(T::default(), params).await
    }

    pub async fn update(
        &self,
        id: i32,
        params: T::PatchParams,
    ) -> Result<T::PatchOutput, ApiError> {
        self.client.patch::<T>(T::default(), id, params).await
    }

    pub async fn delete(&self, id: i32) -> Result<T::DeleteOutput, ApiError> {
        self.client.delete::<T>(T::default(), id).await
    }
}
