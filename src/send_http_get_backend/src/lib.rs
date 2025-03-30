use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};

use ic_cdk_macros::{self, query, update};
use serde::{Serialize, Deserialize};
use serde_json::{self, Value};
use candid::CandidType;

#[derive(Serialize, Deserialize, CandidType)]
struct Location {
    latitude: f64,
    longitude: f64,
}

async fn get_city_coordinates(city: &str) -> Result<(f64, f64), String> {
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
        headers: geocoding_headers,
    };

    match http_request(geocoding_request).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body)
                .expect("Transformed response is not UTF-8 encoded.");
            
            let json: Value = serde_json::from_str(&str_body)
                .expect("Failed to parse JSON response");
            
            if let Some(array) = json.as_array() {
                if let Some(first_result) = array.first() {
                    if let (Some(lat), Some(lon)) = (
                        first_result["lat"].as_str().and_then(|s| s.parse::<f64>().ok()),
                        first_result["lon"].as_str().and_then(|s| s.parse::<f64>().ok()),
                    ) {
                        Ok((lat, lon))
                    } else {
                        Err("Failed to parse coordinates".to_string())
                    }
                } else {
                    Err("No results found for this city".to_string())
                }
            } else {
                Err("Failed to parse geocoding response".to_string())
            }
        }
        Err((r, m)) => Err(format!("Geocoding API error: {:?}, {}", r, m)),
    }
}

async fn get_weather_data_for_coordinates(lat: f64, lon: f64) -> Result<Value, String> {
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m&timezone=auto",
        lat, lon
    );

    let weather_headers = vec![
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "weather_canister".to_string(),
        },
    ];

    let weather_request = CanisterHttpRequestArgument {
        url: weather_url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers: weather_headers,
    };

    match http_request(weather_request).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body)
                .expect("Transformed response is not UTF-8 encoded.");
            
            Ok(serde_json::from_str(&str_body)
                .expect("Failed to parse JSON response"))
        }
        Err((r, m)) => Err(format!("Weather API error: {:?}, {}", r, m)),
    }
}

#[ic_cdk::update]
async fn get_weather_data(city: String) -> String {
    match get_city_coordinates(&city).await {
        Ok((lat, lon)) => {
            match get_weather_data_for_coordinates(lat, lon).await {
                Ok(json) => {
                    if let Some(temp) = json["current"]["temperature_2m"].as_f64() {
                        format!("Temperature in {}: {:.1}°C", city, temp)
                    } else {
                        "Failed to get temperature data".to_string()
                    }
                }
                Err(e) => e,
            }
        }
        Err(e) => e,
    }
}

#[ic_cdk::update]
async fn get_humidity(city: String) -> String {
    match get_city_coordinates(&city).await {
        Ok((lat, lon)) => {
            match get_weather_data_for_coordinates(lat, lon).await {
                Ok(json) => {
                    if let Some(humidity) = json["current"]["relative_humidity_2m"].as_f64() {
                        format!("Humidity in {}: {:.1}%", city, humidity)
                    } else {
                        "Failed to get humidity data".to_string()
                    }
                }
                Err(e) => e,
            }
        }
        Err(e) => e,
    }
}

#[ic_cdk::update]
async fn get_temperature_by_coordinates(location: Location) -> String {
    match get_weather_data_for_coordinates(location.latitude, location.longitude).await {
        Ok(json) => {
            if let Some(temp) = json["current"]["temperature_2m"].as_f64() {
                format!("Temperature at ({}, {}): {:.1}°C", 
                    location.latitude, location.longitude, temp)
            } else {
                "Failed to get temperature data".to_string()
            }
        }
        Err(e) => e,
    }
}

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
