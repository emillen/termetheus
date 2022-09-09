use std::collections::HashMap;

use crate::config::Config;
use reqwest::Error;
use serde::Deserialize ;

use mockall::*;
use mockall::predicate::*;

pub struct Prometheus {
    host_base_path: String,
    query: String,
}

#[automock]
impl Prometheus {
    pub fn new(config: Config) -> Self {
        Self {
            host_base_path: config.base_url().clone(),
            query: config.query().clone(),
        }
    }

    pub async fn get_metrics(&self, from: String, to: String) -> Result<QueryResponse, Error> {
        let client = reqwest::Client::new();
        let response = client.get(
            format!(
                "{}/api/v1/query_range?query={}&start={}&end={}&step=1m",
                &self.host_base_path,
                self.query,
                from,
                to
            )
        ).send().await?;

        let body = response.json::<QueryResponse>().await?;
        Ok(body)
    }
}

#[derive(Clone,Debug, Deserialize)]
pub struct QueryValue (pub f64, pub String);

#[derive(Clone,Debug, Deserialize)]
pub struct QueryResult{
    pub metric: HashMap<String, String>,
    pub values: Vec<QueryValue>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryData {
    pub result_type: String,
    pub result: Vec<QueryResult>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QueryResponse {
    pub status: String,
    pub data: QueryData,
}



#[cfg(test)]
mod test{
    use super::*;
    use mockito::{mock, Matcher, server_address};
    fn mock_query_response() -> String {
        let response = r#"{
            "status": "success",
            "data": {
                "resultType": "matrix",
                "result": [
                    {
                        "metric": {
                            "__name__": "up",
                            "instance": "localhost:9090",
                            "job": "prometheus"
                        },
                        "values": [
                            [
                                1631096190.781,
                                "1"
                            ]
                        ]
                    }
                ]
            }
        }"#;
        response.to_string()
    }

    #[test]
    fn test_deserialize_query_response() {
        let query_response: QueryResponse = serde_json::from_str(&mock_query_response()).unwrap();
        assert_eq!(query_response.status, "success");
        assert_eq!(query_response.data.result[0].metric.get("__name__").unwrap(), "up");
        assert_eq!(query_response.data.result[0].values[0].0, 1631096190.781);
        assert_eq!(query_response.data.result[0].values[0].1, "1");
    }

    #[tokio::test]
    async fn test_get_metrics() {
        let _m = mock("GET", "/api/v1/query_range")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_query_response())
            .match_query(Matcher::Regex(r#"query=up&start=1631096190&end=1631096310"#.to_string()))
            .create();

        let prometheus = Prometheus::new(
            Config::new(
                &vec!["test".to_string(),
                format!("http://{}", server_address()), "up".to_string()]).unwrap()
            );

        let query_response = prometheus.get_metrics("1631096190".to_string(), "1631096310".to_string()).await.unwrap();
        assert_eq!(query_response.status, "success");
        assert_eq!(query_response.data.result[0].metric.get("__name__").unwrap(), "up");
        assert_eq!(query_response.data.result[0].values[0].0, 1631096190.781);
        assert_eq!(query_response.data.result[0].values[0].1, "1");
    }
}
