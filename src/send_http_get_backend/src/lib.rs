//1. IMPORT IC MANAGEMENT CANISTER
//This includes all methods and types needed
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};

use ic_cdk_macros::{self, query, update};
use serde::{Serialize, Deserialize};
use serde_json::{self, Value};

// This struct is used to store the location coordinates
#[derive(Serialize, Deserialize)]
struct Location {
    latitude: f64,
    longitude: f64,
}

//Update method using the HTTPS outcalls feature
#[ic_cdk::update]
async fn get_weather_data(city: String) -> String {
    // First, get coordinates for the city using OpenStreetMap Nominatim API
    let geocoding_url = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1",
        city
    );

    let geocoding_headers = vec![
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "weather_canister".to_string(),
        },
    ];

    let geocoding_request = CanisterHttpRequestArgument {
        url: geocoding_url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers: geocoding_headers.clone(),
    };

    match http_request(geocoding_request).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body)
                .expect("Transformed response is not UTF-8 encoded.");
            
            // Parse the geocoding response
            let json: Value = serde_json::from_str(&str_body)
                .expect("Failed to parse JSON response");
            
            if let Some(array) = json.as_array() {
                if let Some(first_result) = array.first() {
                    if let (Some(lat), Some(lon)) = (
                        first_result["lat"].as_str().and_then(|s| s.parse::<f64>().ok()),
                        first_result["lon"].as_str().and_then(|s| s.parse::<f64>().ok()),
                    ) {
                        // Now get weather data for these coordinates
                        let weather_url = format!(
                            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m&timezone=auto",
                            lat, lon
                        );

                        let weather_request = CanisterHttpRequestArgument {
                            url: weather_url,
                            method: HttpMethod::GET,
                            body: None,
                            max_response_bytes: None,
                            transform: None,
                            headers: geocoding_headers,
                        };

                        match http_request(weather_request).await {
                            Ok((response,)) => {
                                let str_body = String::from_utf8(response.body)
                                    .expect("Transformed response is not UTF-8 encoded.");
                                
                                // Parse the weather response and extract only temperature
                                let json: Value = serde_json::from_str(&str_body)
                                    .expect("Failed to parse JSON response");
                                
                                if let Some(temp) = json["current"]["temperature_2m"].as_f64() {
                                    format!("Temperature in {}: {:.1}°C", city, temp)
                                } else {
                                    "Failed to get temperature data".to_string()
                                }
                            }
                            Err((r, m)) => format!("Weather API error: {:?}, {}", r, m),
                        }
                    } else {
                        "Failed to parse coordinates".to_string()
                    }
                } else {
                    "No results found for this city".to_string()
                }
            } else {
                "Failed to parse geocoding response".to_string()
            }
        }
        Err((r, m)) => format!("Geocoding API error: {:?}, {}", r, m),
    }
}

// Strips all data that is not needed from the original response.
#[query]
fn transform(raw: TransformArgs) -> HttpResponse {
    let headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
        HttpHeader {
            name: "Permissions-Policy".to_string(),
            value: "geolocation=(self)".to_string(),
        },
        HttpHeader {
            name: "Strict-Transport-Security".to_string(),
            value: "max-age=63072000".to_string(),
        },
        HttpHeader {
            name: "X-Frame-Options".to_string(),
            value: "DENY".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];

    let mut res = HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers,
        ..Default::default()
    };

    if res.status == 200 {
        res.body = raw.response.body;
    } else {
        ic_cdk::api::print(format!("Received an error from Open-Meteo: err = {:?}", raw));
    }
    res
}
