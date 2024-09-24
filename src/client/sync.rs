use log::{error, trace};
use reqwest::blocking::Response;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::marker::PhantomData;

use super::{Authenticated, ClientCore, IntoResourceFilter, Unauthenticated};
use crate::endpoints::Endpoint;
use crate::errors::ApiError;
use crate::resources::{ApiResource, Class, Group, Namespace, User};
use crate::types::{BaseUrl, Credentials, FilterOperator, Token};
use crate::QueryFilter;

#[derive(Deserialize, Debug)]
struct DeleteResponse;

#[derive(Debug, Clone)]
pub struct Client<S> {
    http_client: reqwest::blocking::Client,
    base_url: BaseUrl,
    state: S,
}

impl<S> ClientCore for Client<S> {
    fn build_url(&self, endpoint: &Endpoint) -> String {
        format!(
            "{}{}",
            self.base_url.with_trailing_slash(),
            endpoint.trim_start_matches('/')
        )
    }
}

trait ResponseHandler {
    fn check_success(&self, response: Response) -> Result<Response, ApiError>;
}

impl<T> ResponseHandler for Client<T> {
    fn check_success(&self, response: Response) -> Result<Response, ApiError> {
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text()?;
            let error_message = match serde_json::from_str::<Value>(&body) {
                Ok(json) => json["message"]
                    .as_str()
                    .unwrap_or("Error without message.")
                    .to_string(),
                Err(_) => body,
            };
            return Err(ApiError::HttpWithBody {
                status,
                message: error_message,
            });
        }
        Ok(response)
    }
}

impl Client<Unauthenticated> {
    pub fn new(base_url: BaseUrl) -> Self {
        Client {
            http_client: reqwest::blocking::Client::new(),
            base_url,
            state: Unauthenticated,
        }
    }
}

impl Client<Unauthenticated> {
    pub fn login(self, credentials: Credentials) -> Result<Client<Authenticated>, ApiError> {
        let token: Token = self
            .http_client
            .post(&self.build_url(&Endpoint::Login))
            .json(&credentials)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(Client {
            http_client: self.http_client,
            base_url: self.base_url,
            state: Authenticated { token: token.token },
        })
    }

    pub fn login_with_token(self, token: Token) -> Result<Client<Authenticated>, ApiError> {
        let status = self
            .http_client
            .get(self.build_url(&Endpoint::LoginWithToken))
            .header("Authorization", format!("Bearer {}", token.token))
            .send()?;

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

    pub fn request<R: ApiResource, T: Serialize + std::fmt::Debug, U: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        resource: R,
        params: T,
    ) -> Result<Option<U>, ApiError> {
        let endpoint = resource.endpoint();
        let url = self.build_url(&endpoint);

        let request = match method {
            // This is a horrible hack as we have no other traits to work with.
            reqwest::Method::GET => {
                let query = format!("{:?}", params);
                let query = query.strip_prefix('"').unwrap();
                let query = query.strip_suffix('"').unwrap();
                let url = if !query.is_empty() {
                    format!("{}?{}", url, query)
                } else {
                    url
                };
                trace!("GET {}", url);
                self.http_client.get(&url)
            }
            reqwest::Method::POST => {
                trace!("POST {} with {:?}", &url, params);
                self.http_client.post(&url).json(&params)
            }
            reqwest::Method::PATCH => {
                trace!("PATCH {} with {:?}", &url, params);
                self.http_client.patch(&url).json(&params)
            }
            reqwest::Method::DELETE => {
                let url = format!("{}{:?}", url, params);
                trace!("DELETE {}", &url);
                self.http_client.delete(&url)
            }
            _ => return Err(ApiError::UnsupportedHttpOperation(method.to_string())),
        }
        .header("Authorization", format!("Bearer {}", self.state.token));

        let now = std::time::Instant::now();
        let response = request.send()?;
        trace!("Request took {:?}", now.elapsed());
        let response_text = self.check_success(response)?.text()?;
        trace!("Response: {}", response_text);

        if method == reqwest::Method::DELETE {
            if response_text.is_empty() {
                return Ok(None);
            } else {
                error!("Expected empty response, got: {}", response_text);
                return Err(ApiError::DeserializationError(response_text));
            }
        }

        let obj: U = match serde_json::from_str(&response_text) {
            Ok(obj) => obj,
            Err(err) => {
                error!(
                    "Failed to deserialize response: {} Response text: {}",
                    err, response_text
                );
                return Err(ApiError::DeserializationError(response_text));
            }
        };

        Ok(Some(obj))
    }

    pub fn get<R: ApiResource>(
        &self,
        resource: R,
        params: R::GetParams,
    ) -> Result<Vec<R::GetOutput>, ApiError> {
        self.request(reqwest::Method::GET, resource, params)
            .and_then(|opt| opt.ok_or(ApiError::EmptyResult))
    }

    pub fn search<R: ApiResource>(
        &self,
        resource: R,
        params: Vec<QueryFilter>,
    ) -> Result<Vec<R::GetOutput>, ApiError> {
        use crate::types::IntoQueryTuples;
        self.request(reqwest::Method::GET, resource, params.into_query_string())
            .and_then(|opt| opt.ok_or(ApiError::EmptyResult))
    }

    pub fn post<R: ApiResource>(
        &self,
        resource: R,
        params: R::PostParams,
    ) -> Result<R::PostOutput, ApiError> {
        self.request(reqwest::Method::POST, resource, params)
            .and_then(|opt| opt.ok_or(ApiError::EmptyResult))
    }

    pub fn patch<R: ApiResource>(
        &self,
        resource: R,
        id: i32,
        params: R::PatchParams,
    ) -> Result<R::PatchOutput, ApiError> {
        self.request(reqwest::Method::PATCH, resource, (id, params))
            .and_then(|opt| opt.ok_or(ApiError::EmptyResult))
    }

    pub fn delete<R: ApiResource>(&self, resource: R, id: i32) -> Result<(), ApiError> {
        self.request::<_, _, DeleteResponse>(reqwest::Method::DELETE, resource, id)
            .map(|_| ())
    }

    pub fn users(&self) -> Resource<User> {
        Resource::new(self.clone())
    }

    pub fn classes(&self) -> Resource<Class> {
        Resource::new(self.clone())
    }

    pub fn namespaces(&self) -> Resource<Namespace> {
        Resource::new(self.clone())
    }

    pub fn groups(&self) -> Resource<Group> {
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

    pub fn add_filter_name_exact<V: ToString>(self, value: V) -> Self {
        self.add_filter("name", FilterOperator::Equals { is_negated: false }, value)
    }

    pub fn execute(self) -> Result<Vec<T::GetOutput>, ApiError> {
        let params = T::build_params(self.filters);
        let res = self.client.search::<T>(T::default(), params);
        res
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

    pub fn filter(
        &self,
        filter: impl IntoResourceFilter<T>,
    ) -> Result<Vec<T::GetOutput>, ApiError> {
        let params = filter.into_resource_filter();
        self.client.search::<T>(T::default(), params)
    }

    pub fn create(&self, params: T::PostParams) -> Result<T::PostOutput, ApiError> {
        self.client.post::<T>(T::default(), params)
    }

    pub fn update(&self, id: i32, params: T::PatchParams) -> Result<T::PatchOutput, ApiError> {
        self.client.patch::<T>(T::default(), id, params)
    }

    pub fn delete(&self, id: i32) -> Result<(), ApiError> {
        self.client.delete::<T>(T::default(), id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    use yare::parameterized;

    #[parameterized(
        login_foo = { "https://foo.bar.com", Endpoint::Login },
        get_user_foo = { "https://foo.bar.com", Endpoint::Users },
        get_class_foo = { "https://foo.bar.com", Endpoint::Classes },
        login_bar = { "https://bar.baz.com", Endpoint::Login },
        get_user_bar = { "https://bar.baz.com", Endpoint::Users },
        get_class_bar = { "https://bar.baz.com", Endpoint::Classes }

    )]

    fn test_build_url(server: &str, endpoint: Endpoint) {
        let base_url = BaseUrl::from_str(server).unwrap();
        let client = Client::new(base_url.clone());

        assert_eq!(
            client.build_url(&endpoint),
            format!(
                "{}{}",
                base_url.with_trailing_slash(),
                endpoint.trim_start_matches('/')
            )
        );
    }
}
