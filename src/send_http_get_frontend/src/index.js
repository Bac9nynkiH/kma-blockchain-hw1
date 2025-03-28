import { canisterId, createActor } from "../../declarations/send_http_get_backend";
import { AuthClient } from "@dfinity/auth-client";

let actor;

const getWeatherButton = document.getElementById("getWeather");
const weatherDataElement = document.getElementById("weatherData");
const cityInput = document.getElementById("cityInput");

// Initialize the actor
async function init() {
  try {
    const authClient = await AuthClient.create();
    actor = createActor(canisterId, {
      agentOptions: {
        identity: authClient.getIdentity(),
      },
    });
    console.log("Actor initialized successfully");
  } catch (error) {
    console.error("Failed to initialize actor:", error);
    weatherDataElement.textContent = `Error initializing: ${error.message}`;
  }
}

// Initialize when the page loads
init();

// Handle Enter key press in the input field
cityInput.addEventListener("keypress", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    getWeatherButton.click();
  }
});

getWeatherButton.addEventListener("click", async () => {
  const city = cityInput.value.trim();
  if (!city) {
    weatherDataElement.textContent = "Please enter a city name";
    return;
  }

  try {
    weatherDataElement.textContent = "Loading...";
    // Call the canister method with the city parameter
    const result = await actor.get_weather_data(city);
    weatherDataElement.textContent = result;
  } catch (error) {
    console.error("Error fetching weather:", error);
    weatherDataElement.textContent = `Error: ${error.message || "Failed to fetch weather data"}`;
  }
}); 