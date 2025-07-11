use ftl_sdk::{tool, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;
use spin_sdk::http::{send, Method, Request, Response};

#[derive(Deserialize, JsonSchema)]
struct WeatherInput {
    /// City name to get weather for
    location: String,
}

#[derive(Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Deserialize)]
struct GeocodingResult {
    latitude: f64,
    longitude: f64,
    name: String,
}

#[derive(Deserialize)]
struct WeatherResponse {
    current: CurrentWeather,
}

#[derive(Deserialize)]
struct CurrentWeather {
    temperature_2m: f64,
    apparent_temperature: f64,
    relative_humidity_2m: f64,
    wind_speed_10m: f64,
    wind_gusts_10m: f64,
    weather_code: i32,
}

fn get_weather_condition(code: i32) -> &'static str {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 => "Foggy",
        48 => "Depositing rime fog",
        51 => "Light drizzle",
        53 => "Moderate drizzle",
        55 => "Dense drizzle",
        56 => "Light freezing drizzle",
        57 => "Dense freezing drizzle",
        61 => "Slight rain",
        63 => "Moderate rain",
        65 => "Heavy rain",
        66 => "Light freezing rain",
        67 => "Heavy freezing rain",
        71 => "Slight snow fall",
        73 => "Moderate snow fall",
        75 => "Heavy snow fall",
        77 => "Snow grains",
        80 => "Slight rain showers",
        81 => "Moderate rain showers",
        82 => "Violent rain showers",
        85 => "Slight snow showers",
        86 => "Heavy snow showers",
        95 => "Thunderstorm",
        96 => "Thunderstorm with slight hail",
        99 => "Thunderstorm with heavy hail",
        _ => "Unknown",
    }
}

/// Get current weather for a location using Open-Meteo API
#[tool]
async fn weather_rs(input: WeatherInput) -> ToolResponse {
    match fetch_weather(&input.location).await {
        Ok(weather_info) => ToolResponse::text(weather_info),
        Err(e) => ToolResponse::text(format!("Error fetching weather: {}", e)),
    }
}

async fn fetch_weather(location: &str) -> Result<String, String> {
    // First, geocode the location
    let geocoding_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1",
        urlencoding::encode(location)
    );
    
    let geocoding_request = Request::builder()
        .method(Method::Get)
        .uri(geocoding_url)
        .build();
    
    let geocoding_response: Response = send(geocoding_request).await.map_err(|e| format!("Failed to fetch geocoding data: {}", e))?;
    
    if *geocoding_response.status() != 200 {
        return Err(format!("Geocoding API returned status: {}", geocoding_response.status()));
    }
    
    let geocoding_body = geocoding_response.body();
    let geocoding_data: GeocodingResponse = serde_json::from_slice(&geocoding_body)
        .map_err(|e| format!("Failed to parse geocoding response: {}", e))?;
    
    let geocoding_result = geocoding_data.results
        .and_then(|r| r.into_iter().next())
        .ok_or_else(|| format!("Location '{}' not found", location))?;
    
    // Now fetch the weather data
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,wind_gusts_10m,weather_code",
        geocoding_result.latitude,
        geocoding_result.longitude
    );
    
    let weather_request = Request::builder()
        .method(Method::Get)
        .uri(weather_url)
        .build();
    
    let weather_response: Response = send(weather_request).await.map_err(|e| format!("Failed to fetch weather data: {}", e))?;
    
    if *weather_response.status() != 200 {
        return Err(format!("Weather API returned status: {}", weather_response.status()));
    }
    
    let weather_body = weather_response.body();
    let weather_data: WeatherResponse = serde_json::from_slice(&weather_body)
        .map_err(|e| format!("Failed to parse weather response: {}", e))?;
    
    let current = &weather_data.current;
    let conditions = get_weather_condition(current.weather_code);
    
    Ok(format!(
        "Weather in {}:\n\
        Temperature: {}°C (feels like {}°C)\n\
        Conditions: {}\n\
        Humidity: {}%\n\
        Wind: {} km/h (gusts up to {} km/h)",
        geocoding_result.name,
        current.temperature_2m,
        current.apparent_temperature,
        conditions,
        current.relative_humidity_2m,
        current.wind_speed_10m,
        current.wind_gusts_10m
    ))
}

// Add URL encoding since it's not in spin-sdk
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                ' ' => "+".to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}