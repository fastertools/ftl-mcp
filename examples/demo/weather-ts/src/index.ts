import { createTool, ToolResponse } from 'ftl-sdk'
import { z } from 'zod'

const WeatherInputSchema = z.object({
  location: z.string().describe('City name')
})

type WeatherInput = z.infer<typeof WeatherInputSchema>

interface GeocodingResponse {
  results: {
    latitude: number;
    longitude: number;
    name: string;
  }[];
}

interface WeatherResponse {
  current: {
    time: string;
    temperature_2m: number;
    apparent_temperature: number;
    relative_humidity_2m: number;
    wind_speed_10m: number;
    wind_gusts_10m: number;
    weather_code: number;
  };
}

const getWeatherCondition = (code: number): string => {
  const conditions: Record<number, string> = {
    0: 'Clear sky',
    1: 'Mainly clear',
    2: 'Partly cloudy',
    3: 'Overcast',
    45: 'Foggy',
    48: 'Depositing rime fog',
    51: 'Light drizzle',
    53: 'Moderate drizzle',
    55: 'Dense drizzle',
    56: 'Light freezing drizzle',
    57: 'Dense freezing drizzle',
    61: 'Slight rain',
    63: 'Moderate rain',
    65: 'Heavy rain',
    66: 'Light freezing rain',
    67: 'Heavy freezing rain',
    71: 'Slight snow fall',
    73: 'Moderate snow fall',
    75: 'Heavy snow fall',
    77: 'Snow grains',
    80: 'Slight rain showers',
    81: 'Moderate rain showers',
    82: 'Violent rain showers',
    85: 'Slight snow showers',
    86: 'Heavy snow showers',
    95: 'Thunderstorm',
    96: 'Thunderstorm with slight hail',
    99: 'Thunderstorm with heavy hail',
  };
  return conditions[code] || 'Unknown';
}

const weather = createTool<WeatherInput>({
  metadata: {
    name: 'weather_ts',
    title: 'Weather Tool (TypeScript)',
    description: 'Get current weather for a location using Open-Meteo API',
    inputSchema: z.toJSONSchema(WeatherInputSchema)
  },
  handler: async (input) => {
    try {
      const geocodingUrl = `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(input.location)}&count=1`;
      const geocodingResponse = await fetch(geocodingUrl);
      const geocodingData = (await geocodingResponse.json()) as GeocodingResponse;

      if (!geocodingData.results?.[0]) {
        return ToolResponse.text(`Location '${input.location}' not found`);
      }

      const { latitude, longitude, name } = geocodingData.results[0];

      const weatherUrl = `https://api.open-meteo.com/v1/forecast?latitude=${latitude}&longitude=${longitude}&current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,wind_gusts_10m,weather_code`;

      const response = await fetch(weatherUrl);
      const data = (await response.json()) as WeatherResponse;

      const result = {
        temperature: data.current.temperature_2m,
        feelsLike: data.current.apparent_temperature,
        humidity: data.current.relative_humidity_2m,
        windSpeed: data.current.wind_speed_10m,
        windGust: data.current.wind_gusts_10m,
        conditions: getWeatherCondition(data.current.weather_code),
        location: name,
      };

      return ToolResponse.text(`Weather in ${result.location}:
Temperature: ${result.temperature}°C (feels like ${result.feelsLike}°C)
Conditions: ${result.conditions}
Humidity: ${result.humidity}%
Wind: ${result.windSpeed} km/h (gusts up to ${result.windGust} km/h)`);
    } catch (error) {
      return ToolResponse.text(`Error fetching weather: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }
})

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(weather(event.request))
})