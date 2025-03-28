import { canisterId, createActor } from "../../declarations/send_http_get_backend";
import { AuthClient } from "@dfinity/auth-client";

let actor;
let currentCity = null;
let currentTemperature = null;
let pinnedTemperatures = [];

const getWeatherButton = document.getElementById("getWeather");
const weatherDataElement = document.getElementById("weatherData");
const cityInput = document.getElementById("cityInput");
const pinButton = document.getElementById("pinTemperature");
const pinnedList = document.getElementById("pinnedList");

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

// Update pin button state based on current temperature
function updatePinButton() {
  pinButton.disabled = !currentTemperature || pinnedTemperatures.length >= 5;
}

// Create a pinned temperature element
function createPinnedTemperatureElement(city, temperature) {
  const div = document.createElement("div");
  div.className = "pinned-temperature";
  div.innerHTML = `
    <span class="temperature">${city}: ${temperature}°C</span>
    <div class="actions">
      <button class="refresh-btn">Refresh</button>
      <div class="loading-spinner"></div>
      <button class="delete-btn">Delete</button>
    </div>
  `;

  // Add refresh functionality
  div.querySelector(".refresh-btn").addEventListener("click", async () => {
    const actionsDiv = div.querySelector(".actions");
    actionsDiv.classList.add("loading");
    
    try {
      const result = await actor.get_weather_data(city);
      const temp = result.match(/\d+\.?\d*/)[0];
      div.querySelector(".temperature").textContent = `${city}: ${temp}°C`;
    } catch (error) {
      console.error("Error refreshing temperature:", error);
      div.querySelector(".temperature").textContent = `${city}: Error refreshing`;
    } finally {
      actionsDiv.classList.remove("loading");
    }
  });

  // Add delete functionality
  div.querySelector(".delete-btn").addEventListener("click", () => {
    pinnedTemperatures = pinnedTemperatures.filter(p => p.city !== city);
    div.remove();
    updatePinButton();
  });

  return div;
}

// Update the pinned temperatures list
function updatePinnedList() {
  pinnedList.innerHTML = "";
  pinnedTemperatures.forEach(({ city, temperature }) => {
    pinnedList.appendChild(createPinnedTemperatureElement(city, temperature));
  });
}

// Handle pin button click
pinButton.addEventListener("click", () => {
  if (currentTemperature && currentCity && pinnedTemperatures.length < 5) {
    pinnedTemperatures.push({ city: currentCity, temperature: currentTemperature });
    updatePinnedList();
    updatePinButton();
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
    const result = await actor.get_weather_data(city);
    weatherDataElement.textContent = result;
    
    // Extract temperature from result
    const tempMatch = result.match(/\d+\.?\d*/);
    if (tempMatch) {
      currentTemperature = tempMatch[0];
      currentCity = city;
      updatePinButton();
    }
  } catch (error) {
    console.error("Error fetching weather:", error);
    weatherDataElement.textContent = `Error: ${error.message || "Failed to fetch weather data"}`;
    currentTemperature = null;
    currentCity = null;
    updatePinButton();
  }
}); 