use serde::Deserialize;

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub enum Widget {
    #[serde(rename = "timeseries")]
    TimeSeries {
        name: String,
        query: String,
        source: String,
    },
    #[serde(rename = "stat")]
    Stat {
        name: String,
        query: String,
        source: String,
    },
}

#[derive(Clone, Debug, Deserialize)]
pub struct Row {
    pub widgets: Vec<Widget>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Dashboard {
    pub name: String,
    pub rows: Vec<Row>,
}

impl Dashboard {
    pub fn new(file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::read_to_string(file)?;
        let dashboard: Dashboard = serde_yaml::from_str(&file)?;
        Ok(dashboard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn test_deserialize_dashboard() {
        let dashboard = r#"
        name: Example Dashboard
        rows:
          - widgets:
              - stat:
                  name: amount of online hosts
                  query: count(up{} == 1)
                  source: http://localhost:9090
              - stat:
                  name: amount of offline hosts
                  query: count(up{} == 0)
                  source: https://localhost:9090
          - widgets:
              - timeseries:
                  query: up
                  source: http://localhost:9090
                  name: hosts up timeline
        "#;
        let dashboard: Dashboard = serde_yaml::from_str(dashboard).unwrap();
        assert_eq!(dashboard.name, "Example Dashboard");
        assert_eq!(dashboard.rows.len(), 2);
        assert_eq!(dashboard.rows[0].widgets.len(), 2);
        assert_eq!(dashboard.rows[1].widgets.len(), 1);
        assert_eq!(
            dashboard.rows[0].widgets[0],
            Widget::Stat {
                name: "amount of online hosts".to_string(),
                query: "count(up{} == 1)".to_string(),
                source: "http://localhost:9090".to_string(),
            }
        );
        assert_eq!(
            dashboard.rows[0].widgets[1],
            Widget::Stat {
                name: "amount of offline hosts".to_string(),
                query: "count(up{} == 0)".to_string(),
                source: "https://localhost:9090".to_string(),
            }
        );
        assert_eq!(
            dashboard.rows[1].widgets[0],
            Widget::TimeSeries {
                name: "hosts up timeline".to_string(),
                query: "up".to_string(),
                source: "http://localhost:9090".to_string(),
            }
        );
    }
}
