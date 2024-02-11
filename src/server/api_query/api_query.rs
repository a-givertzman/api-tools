#![allow(non_snake_case)]

use log::{debug, warn};
use serde::{Serialize, Deserialize};

use crate::{
    error::api_error::ApiError, 
    server::api_query::{api_query_error::ApiQueryError, api_query_executable::ApiQueryExecutable, api_query_python::ApiQueryPython, api_query_sql::ApiQuerySql, api_query_type::{ApiQueryType, ApiQueryTypeName}}, 
};

///
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiQuery {
    authToken: String,
    id: String,
    query: ApiQueryType,
    srcQuery: String, 
    pub keepAlive: bool,
    pub debug: bool,
}
impl ApiQuery {
    ///
    /// 
    pub fn authToken(&self) -> String {
        self.authToken.clone()
    }
    ///
    /// 
    pub fn id(&self) -> String {
        self.id.clone()
    }
    ///
    /// 
    pub fn query(&self) -> ApiQueryType {
        self.query.clone()
    }
    ///
    /// Returns original query string
    pub fn srcQuery(&self, debug: bool) -> String {
        if debug {
            self.srcQuery.clone()
        } else {
            "".to_string()
        }
    }
    ///
    /// 
    pub fn new(auth_token: String, id: String, query: ApiQueryType, srcQuery: impl Into<String>, keepAlive: bool, debug: bool) -> Self {
        Self { 
            authToken: auth_token, 
            id, 
            query, 
            srcQuery: srcQuery.into(), 
            keepAlive, 
            debug,
        }
    }
    ///
    fn parseApiQuerySql(srcQuery: &str, json: serde_json::Value, auth_token: String, id: String, keepAlive: bool, debug: bool) -> ApiQuery {
        debug!("[ApiQuery.parseApiQuerySql] detected: {}", ApiQueryTypeName::Sql.value());
        match ApiQuerySql::fromJson(json[ApiQueryTypeName::Sql.value()].clone()) {
            Ok(apiQuerySql) => {
                ApiQuery::new(
                    auth_token,
                    id,
                    ApiQueryType::Sql( apiQuerySql ),
                    srcQuery,
                    keepAlive,
                    debug,
                )
            },
            Err(err) => {
                ApiQuery::new(
                    auth_token,
                    id,
                    ApiQueryType::Error( ApiQueryError::new(err) ),
                    srcQuery,
                    keepAlive,
                    debug,
                )
            },
        }
    }
    ///
    fn parseApiQueryPython(srcQuery: &str, json: serde_json::Value, auth_token: String, id: String, keepAlive: bool, debug: bool) -> ApiQuery {
        debug!("ApiQuery.fromBytes | detected: {}", ApiQueryTypeName::Python.value());
        match ApiQueryPython::fromJson(json[ApiQueryTypeName::Python.value()].clone()) {
            Ok(apiQueryPython) => {
                ApiQuery::new(
                    auth_token,
                    id,
                    ApiQueryType::Python( apiQueryPython ),
                    srcQuery,
                    keepAlive,
                    debug,
                )
            },
            Err(err) => {
                ApiQuery::new(
                    auth_token,
                    id,
                    ApiQueryType::Error( ApiQueryError::new(err) ),
                    srcQuery,
                    keepAlive,
                    debug,
                )
            },
        }
    }
    ///
    fn parseApiQueryExecutable(srcQuery: &str, json: serde_json::Value, auth_token: String, id: String, keepAlive: bool, debug: bool) -> ApiQuery {
        debug!("ApiQuery.fromBytes | detected: {}", ApiQueryTypeName::Executable.value());
        match ApiQueryExecutable::fromJson(json[ApiQueryTypeName::Executable.value()].clone()) {
            Ok(apiQueryExecutable) => {
                ApiQuery::new(
                    auth_token,
                    id,
                    ApiQueryType::Executable( apiQueryExecutable ),
                    srcQuery,
                    keepAlive,
                    debug,
                )
            },
            Err(err) => {
                ApiQuery::new(
                    auth_token,
                    id,
                    ApiQueryType::Error( ApiQueryError::new(err) ),
                    srcQuery,
                    keepAlive,
                    debug,
                )
            },
        }
    }
    ///
    ///
    fn parseQueryTypeName(query: &serde_json::map::Map<String, serde_json::Value>) -> Result<ApiQueryTypeName, ApiError> {
        let mut queries = 0;
        let mut queryType = ApiQueryTypeName::Unknown;
        if query.contains_key(ApiQueryTypeName::Sql.value()) {
            queries += 1;
            queryType = ApiQueryTypeName::Sql
        }
        if query.contains_key(ApiQueryTypeName::Python.value()) {
            queries += 1;
            queryType = ApiQueryTypeName::Python;
        }
        if query.contains_key(ApiQueryTypeName::Executable.value()) {
            queries += 1;
            queryType = ApiQueryTypeName::Executable
        }
        match queries.cmp(&1) {
            std::cmp::Ordering::Less => Ok(queryType),
            std::cmp::Ordering::Equal => Ok(queryType),
            std::cmp::Ordering::Greater => {
                let details = format!("ApiQuery.fromBytes | Unable to perform multiservice request: {:?}", query);
                warn!("{}", details);
                Err(
                    ApiError::new(
                        format!("API Service - Unable to perform multiservice request: {:?}", query), 
                        details,
                    )
                )
            },
        }
    }
    ///
    /// builds ApiQuery from bytes
    pub fn fromBytes(bytes: Vec<u8>) -> Self {
        let mut auth_token = "Unknown".to_string();
        let mut id = "Unknown".to_string();
        let mut keepAlive = false;
        let mut debug = true;
        let refBytes = &bytes;
        match serde_json::from_slice::<serde_json::Value>(refBytes) {
            Ok(json) => {
                match &json.as_object() {
                    Some(queryMap) => {
                        let mut errors = vec![];
                        match queryMap.getValue("authToken") {
                            Ok(value) => auth_token = value,
                            Err(err) => errors.push(err),
                        };
                        match queryMap.getValue("id") {
                            Ok(value) => id = value,
                            Err(err) => errors.push(err),
                        };
                        match queryMap.getValue("keepAlive") {
                            Ok(value) => {
                                debug!("ApiQuery.fromBytes | keep-alive detected");
                                keepAlive = value;
                            },
                            Err(_) => {},
                        };
                        match queryMap.getValue("debug") {
                            Ok(value) => {
                                debug!("ApiQuery.fromBytes | debug detected");
                                debug = value;
                            },
                            Err(_) => debug = false,
                        };
                        match errors.get(0) {
                            Some(details) => {
                                ApiQuery {
                                    authToken: auth_token,
                                    id,
                                    query: ApiQueryType::Error( ApiQueryError::new(
                                        // json.clone(),
                                        ApiError::new(
                                            format!("API Service - invalid query: {:?}", json), 
                                            format!("ApiQuery.fromBytes | errors: {:?}", details), 
                                        ),
                                    )),
                                    srcQuery: json.to_string(),
                                    keepAlive,
                                    debug,
                                }
                            },
                            None => {
                                debug!("ApiQuery.fromBytes | obj: {:?}", queryMap);
                                match Self::parseQueryTypeName(&queryMap) {
                                    Ok(queryType) => match queryType {
                                        ApiQueryTypeName::Sql => Self::parseApiQuerySql(&json.to_string(), json, auth_token, id, keepAlive, debug),
                                        ApiQueryTypeName::Python => Self::parseApiQueryPython(&json.to_string(), json, auth_token, id, keepAlive, debug),
                                        ApiQueryTypeName::Executable => Self::parseApiQueryExecutable(&json.to_string(), json, auth_token, id, keepAlive, debug),
                                        ApiQueryTypeName::Unknown => ApiQuery {
                                            authToken: auth_token,
                                            id,
                                            query: ApiQueryType::Unknown,
                                            srcQuery: json.to_string(), 
                                            keepAlive,
                                            debug,
                                        },
                                    },
                                    Err(err) => {
                                        ApiQuery {
                                            authToken: auth_token,
                                            id,
                                            query: ApiQueryType::Error( ApiQueryError::new(err)),
                                            srcQuery: json.to_string(),
                                            keepAlive,
                                            debug,
                                        }
                                    },
                                }
                            },
                        }
                    },
                    None => {
                        let details = format!("ApiQuery.fromBytes | json parsing error: type Map not found in json: {:?}", json.to_string());
                        warn!("{}", details);
                        ApiQuery {
                            authToken: auth_token,
                            id,
                            query: ApiQueryType::Error( ApiQueryError::new(
                                // json.clone(), 
                                ApiError::new(
                                    format!("API Service - invalid query: {:?}", json), 
                                    details,
                                ),
                            )),
                            srcQuery: json.to_string(),
                            keepAlive,
                            debug,
                        }
                    },
                }
            },
            Err(err) => {
                let details = format!("ApiQuery.fromBytes | json parsing error: {:?}", err);
                let default = refBytes.iter().map(|v| v.to_string()).reduce(|i, v| i + "," + &v).unwrap_or(String::new());
                let queryString = String::from_utf8(refBytes.to_owned()).unwrap_or(default);
                warn!("{} \n\tin query: {}", details, queryString);
                ApiQuery {
                    authToken: auth_token,
                    id,
                    query: ApiQueryType::Error( ApiQueryError::new(
                        // json!(queryString), 
                        ApiError::new(
                            format!("API Service - invalid query: {:?}", queryString), 
                            details,
                        )
                    )),
                    srcQuery: queryString,
                    keepAlive,
                    debug,
                }
            },
        }
    }
}


trait GetJsonObjValue<T> {
    fn getValue(&self, key: &str) -> Result<T, String>;
}

impl GetJsonObjValue<String> for serde_json::map::Map<String, serde_json::Value> {
    fn getValue(&self, key: &str) -> Result<String, String> {
        let msg = format!("ApiQuery.fromBytes | field '{}' of type {:?} not found or invalid content", &key, "String");
        match self.get(key) {
            Some(jsonValue) => {
                if let serde_json::Value::String(value) = jsonValue {
                    Ok(value.to_string())
                } else {
                    Err(msg)
                }
            },
            None => {
                Err(msg)
            },
        }
    }
}

impl GetJsonObjValue<bool> for serde_json::map::Map<String, serde_json::Value> {
    fn getValue(&self, key: &str) -> Result<bool, String> {
        let msg = format!("ApiQuery.fromBytes | field '{}' of type {:?} not found or invalid content", &key, "String");
        match self.get(key) {
            Some(jsonValue) => {
                if let serde_json::Value::Bool(value) = jsonValue {
                    Ok(*value)
                } else {
                    Err(msg)
                }
            },
            None => {
                Err(msg)
            },
        }
    }
}

