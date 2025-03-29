use std::{env, sync::Arc};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use context_server::{Tool, ToolContent, ToolExecutor};
use http_client::{HttpClient, Request, RequestBuilderExt, ResponseAsyncBodyExt};
use serde_json::{Value, json};

pub struct PlanTripTool {
    http_client: Arc<dyn HttpClient>,
}

impl PlanTripTool {
    pub fn new(http_client: Arc<dyn HttpClient>) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl ToolExecutor for PlanTripTool {
    async fn execute(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        log::debug!("Executing PlanTripTool");
        let args = arguments.ok_or_else(|| anyhow!("Missing arguments"))?;

        let fly_from = args
            .get("fly_from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid fly_from parameter"))?;

        let fly_to = args
            .get("fly_to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid fly_to parameter"))?;

        let date_from = args
            .get("date_from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid date_from parameter"))?;

        let date_to = args
            .get("date_to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid date_to parameter"))?;

        let return_from = args.get("return_from").and_then(|v| v.as_str());
        let return_to = args.get("return_to").and_then(|v| v.as_str());
        let adults = args.get("adults").and_then(|v| v.as_u64()).unwrap_or(1);
        let children = args.get("children").and_then(|v| v.as_u64()).unwrap_or(0);
        let infants = args.get("infants").and_then(|v| v.as_u64()).unwrap_or(0);
        let selected_cabins = args
            .get("selected_cabins")
            .and_then(|v| v.as_str())
            .unwrap_or("M");
        let curr = args.get("curr").and_then(|v| v.as_str()).unwrap_or("EUR");
        let max_stopovers = args
            .get("max_stopovers")
            .and_then(|v| v.as_u64())
            .unwrap_or(2);
        let sort = args.get("sort").and_then(|v| v.as_str()).unwrap_or("price");
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5);

        // Build the URL with query parameters
        let mut url = format!(
            "https://api.tequila.kiwi.com/v2/search?fly_from={}&fly_to={}&date_from={}&date_to={}&adults={}&children={}&infants={}&selected_cabins={}&curr={}&max_stopovers={}&sort={}&limit={}",
            fly_from,
            fly_to,
            date_from,
            date_to,
            adults,
            children,
            infants,
            selected_cabins,
            curr,
            max_stopovers,
            sort,
            limit
        );

        // Add optional return parameters if provided
        if let Some(return_from_val) = return_from {
            url.push_str(&format!("&return_from={}", return_from_val));
        }
        if let Some(return_to_val) = return_to {
            url.push_str(&format!("&return_to={}", return_to_val));
        }

        log::info!("Searching for flights from {} to {}", fly_from, fly_to);

        // Get API key from environment
        let api_key = env::var("KIWI_API_KEY").map_err(|_| {
            log::error!("KIWI_API_KEY not set in environment");
            anyhow!("KIWI_API_KEY not set in environment")
        })?;

        // Make the request to Kiwi API
        let response = self
            .http_client
            .send(
                Request::builder()
                    .method("GET")
                    .uri(url)
                    .header("apikey", api_key)
                    .header("Accept", "application/json")
                    .end()?,
            )
            .await?;

        // Parse the response
        let response_body: Value = response.json().await.map_err(|err| {
            log::error!("Failed to parse API response: {}", err);
            anyhow!("Failed to parse API response: {}", err)
        })?;

        // Format the flight results
        let formatted_results = self.format_flight_results(&response_body, curr)?;

        Ok(vec![ToolContent::Text {
            text: formatted_results,
        }])
    }

    fn to_tool(&self) -> Tool {
        Tool {
            name: "plan_trip".into(),
            description: Some(
                "Search for flights between destinations with flexible date options".into(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "fly_from": {
                        "type": "string",
                        "description": "IATA code of departure location (e.g., 'LHR', 'NYC', 'UK')"
                    },
                    "fly_to": {
                        "type": "string",
                        "description": "IATA code of arrival location"
                    },
                    "date_from": {
                        "type": "string",
                        "description": "Departure date in format dd/mm/yyyy"
                    },
                    "date_to": {
                        "type": "string",
                        "description": "Latest departure date in format dd/mm/yyyy"
                    },
                    "return_from": {
                        "type": "string",
                        "description": "Return departure date in format dd/mm/yyyy (for round trips)"
                    },
                    "return_to": {
                        "type": "string",
                        "description": "Latest return departure date in format dd/mm/yyyy (for round trips)"
                    },
                    "adults": {
                        "type": "integer",
                        "description": "Number of adult passengers"
                    },
                    "children": {
                        "type": "integer",
                        "description": "Number of child passengers"
                    },
                    "infants": {
                        "type": "integer",
                        "description": "Number of infant passengers"
                    },
                    "selected_cabins": {
                        "type": "string",
                        "description": "Cabin class: M (economy), W (economy premium), C (business), F (first class)",
                        "enum": ["M", "W", "C", "F"]
                    },
                    "curr": {
                        "type": "string",
                        "description": "Currency for prices (e.g., EUR, USD, GBP)"
                    },
                    "max_stopovers": {
                        "type": "integer",
                        "description": "Maximum number of stopovers"
                    },
                    "sort": {
                        "type": "string",
                        "description": "Sort results by (price, duration, date, quality)",
                        "enum": ["price", "duration", "date", "quality"]
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return"
                    }
                },
                "required": ["fly_from", "fly_to", "date_from", "date_to"]
            }),
        }
    }
}

impl PlanTripTool {
    fn format_flight_results(&self, response: &Value, currency: &str) -> Result<String> {
        if let Some(data) = response.get("data").and_then(|d| d.as_array()) {
            if data.is_empty() {
                return Ok(String::from("No flights found matching your criteria."));
            }

            let mut result = format!("Found {} flights matching your criteria:\n\n", data.len());

            for (i, flight) in data.iter().enumerate() {
                let price = flight.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0);
                let from = flight
                    .get("cityFrom")
                    .and_then(|c| c.as_str())
                    .unwrap_or("Unknown");
                let to = flight
                    .get("cityTo")
                    .and_then(|c| c.as_str())
                    .unwrap_or("Unknown");
                let from_code = flight
                    .get("flyFrom")
                    .and_then(|c| c.as_str())
                    .unwrap_or("???");
                let to_code = flight
                    .get("flyTo")
                    .and_then(|c| c.as_str())
                    .unwrap_or("???");

                // Format dates from UTC to local readable format
                let departure = flight
                    .get("local_departure")
                    .and_then(|d| d.as_str())
                    .unwrap_or("Unknown");
                let arrival = flight
                    .get("local_arrival")
                    .and_then(|d| d.as_str())
                    .unwrap_or("Unknown");

                // Parse and format the dates
                let departure_formatted =
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(departure) {
                        dt.format("%d %b %Y, %H:%M").to_string()
                    } else {
                        departure.to_string()
                    };

                let arrival_formatted =
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(arrival) {
                        dt.format("%d %b %Y, %H:%M").to_string()
                    } else {
                        arrival.to_string()
                    };

                // Format duration
                let duration_minutes = flight
                    .get("duration")
                    .and_then(|d| d.get("total"))
                    .and_then(|t| t.as_i64())
                    .unwrap_or(0);
                let hours = duration_minutes / 60;
                let minutes = duration_minutes % 60;

                // Get airlines
                let airlines = flight
                    .get("airlines")
                    .and_then(|a| a.as_array())
                    .map(|airlines| {
                        airlines
                            .iter()
                            .filter_map(|a| a.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_else(|| "Unknown".to_string());

                // Stopovers information
                let stops = flight
                    .get("route")
                    .and_then(|r| r.as_array())
                    .map(|routes| routes.len() - 1)
                    .unwrap_or(0);

                let stop_description = match stops {
                    0 => "Direct flight".to_string(),
                    1 => "1 stopover".to_string(),
                    n => format!("{} stopovers", n),
                };

                // Baggage allowance
                let baggage_info = if let Some(bags_price) = flight.get("bags_price") {
                    let first_bag_price =
                        bags_price.get("1").and_then(|p| p.as_f64()).unwrap_or(0.0);
                    format!("First checked bag: {:.2} {}", first_bag_price, currency)
                } else {
                    "Baggage information not available".to_string()
                };

                // Get booking deep link
                let deep_link = flight
                    .get("deep_link")
                    .and_then(|d| d.as_str())
                    .unwrap_or("Booking link not available");

                // Add flight details to result
                result.push_str(&format!(
                    "Flight {}: {} ({}) → {} ({})\n",
                    i + 1,
                    from,
                    from_code,
                    to,
                    to_code
                ));
                result.push_str(&format!("Price: {:.2} {}\n", price, currency));
                result.push_str(&format!("Departure: {}\n", departure_formatted));
                result.push_str(&format!("Arrival: {}\n", arrival_formatted));
                result.push_str(&format!("Duration: {}h {}m\n", hours, minutes));
                result.push_str(&format!("Airline(s): {}\n", airlines));
                result.push_str(&format!("Stops: {}\n", stop_description));
                result.push_str(&format!("{}\n", baggage_info));
                result.push_str(&format!("Booking link: {}\n", deep_link));

                // Add route details for flights with stopovers
                if stops > 0 {
                    if let Some(routes) = flight.get("route").and_then(|r| r.as_array()) {
                        result.push_str("Route details:\n");
                        for (j, route) in routes.iter().enumerate() {
                            let route_from = route
                                .get("cityFrom")
                                .and_then(|c| c.as_str())
                                .unwrap_or("Unknown");
                            let route_to = route
                                .get("cityTo")
                                .and_then(|c| c.as_str())
                                .unwrap_or("Unknown");
                            let route_airline = route
                                .get("airline")
                                .and_then(|a| a.as_str())
                                .unwrap_or("Unknown");

                            result.push_str(&format!(
                                "  Leg {}: {} → {} ({})\n",
                                j + 1,
                                route_from,
                                route_to,
                                route_airline
                            ));
                        }
                    }
                }

                // Add a separator between flights
                if i < data.len() - 1 {
                    result.push_str("\n---\n\n");
                }
            }

            Ok(result)
        } else {
            log::warn!("Unexpected API response format");
            Ok(String::from(
                "Unable to retrieve flight information. The API response was in an unexpected format.",
            ))
        }
    }
}
