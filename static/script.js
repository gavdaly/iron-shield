let clockInterval;

document.addEventListener("DOMContentLoaded", () => {
  initializeTheme();
  setClock();
  setupThemeToggle();
});

function initializeTheme() {
  // Check for saved theme preference or respect OS preference
  const savedTheme = localStorage.getItem("theme");
  const osPrefersDark = window.matchMedia(
    "(prefers-color-scheme: dark)",
  ).matches;

  // Set the theme based on preference
  if (savedTheme === "light" || (!savedTheme && !osPrefersDark)) {
    document.body.classList.add("light-theme");
    updateThemeToggleIcon(true); // Light theme
  } else {
    document.body.classList.remove("light-theme");
    updateThemeToggleIcon(false); // Dark theme
  }
}

function setupThemeToggle() {
  const themeToggle = document.getElementById("theme-toggle");

  themeToggle.addEventListener("click", () => {
    document.body.classList.toggle("light-theme");

    // Save the preference
    const isLightTheme = document.body.classList.contains("light-theme");
    localStorage.setItem("theme", isLightTheme ? "light" : "dark");

    // Update icon
    updateThemeToggleIcon(isLightTheme);
  });
}

function updateThemeToggleIcon(isLightTheme) {
  const themeToggle = document.getElementById("theme-toggle");
  themeToggle.innerHTML = isLightTheme ? "🌙" : "☀️";
}

function setClock() {
  let timeElement = document.getElementById("time");
  if (timeElement) {
    let data = timeElement.dataset.format;

    // Immediately update the time
    updateTimeAndBackground(timeElement, data);

    // Then update every second
    clockInterval = setInterval(() => {
      updateTimeAndBackground(timeElement, data);
    }, 1000);
  }
}

function updateTimeAndBackground(timeElement, data) {
  let now = new Date();
  let hours = now.getHours();
  let minutes = ("0" + now.getMinutes()).slice(-2);

  // Update time display
  if (data === "12hour") {
    let displayHours = hours % 12;
    if (displayHours === 0) displayHours = 12;
    timeElement.innerText = `${displayHours}:${minutes}`;
  } else {
    timeElement.innerText = `${hours}:${minutes}`;
  }

  // Update background based on time of day
  updateTimeOfDayBackground(now);
}

function updateTimeOfDayBackground(date) {
  const body = document.body;
  const hour = date.getHours();

  // Remove existing time classes
  body.classList.remove(
    "time-morning",
    "time-afternoon",
    "time-evening",
    "time-night",
  );

  // Add appropriate class based on time of day
  if (hour >= 5 && hour < 12) {
    // Morning: 5am - 12pm
    body.classList.add("time-morning");
  } else if (hour >= 12 && hour < 17) {
    // Afternoon: 12pm - 5pm
    body.classList.add("time-afternoon");
  } else if (hour >= 17 && hour < 21) {
    // Evening: 5pm - 9pm
    body.classList.add("time-evening");
  } else {
    // Night: 9pm - 5am
    body.classList.add("time-night");
  }
}

// Check uptime for all sites
function checkUptimeForAllSites() {
  const siteLinks = document.querySelectorAll(".site-card a");
  siteLinks.forEach((link) => {
    const url = link.getAttribute("href");
    const uptimeElement = link.parentElement.querySelector(".uptime");

    // Set initial state as loading
    uptimeElement.className = "uptime loading";
    uptimeElement.innerHTML = "<span>•••</span>";

    // Perform a simple fetch to check if the site is reachable
    // Using a proxy to avoid CORS issues
    const proxyUrl =
      "https://api.allorigins.win/raw?url=" + encodeURIComponent(url);

    // Timeout after 5 seconds
    const timeout = setTimeout(() => {
      uptimeElement.className = "uptime down";
      uptimeElement.innerHTML = "<span>●</span> Down";
    }, 5000);

    fetch(proxyUrl, { method: "HEAD", mode: "cors" })
      .then((response) => {
        clearTimeout(timeout);
        if (response.ok) {
          uptimeElement.className = "uptime up";
          uptimeElement.innerHTML = "<span>●</span> Up";
        } else {
          uptimeElement.className = "uptime down";
          uptimeElement.innerHTML = "<span>●</span> Down";
        }
      })
      .catch(() => {
        clearTimeout(timeout);
        uptimeElement.className = "uptime down";
        uptimeElement.innerHTML = "<span>●</span> Down";
      });
  });
}

// Listen for OS theme preference changes
window
  .matchMedia("(prefers-color-scheme: dark)")
  .addEventListener("change", (e) => {
    // Only update if user hasn't manually set a preference
    if (!localStorage.getItem("theme")) {
      if (e.matches) {
        document.body.classList.remove("light-theme");
        updateThemeToggleIcon(false);
      } else {
        document.body.classList.add("light-theme");
        updateThemeToggleIcon(true);
      }
    }
  });

window.addEventListener("beforeunload", () => {
  clearInterval(clockInterval);
});

// Initialize uptime checks after DOM is loaded and themes are set
document.addEventListener("DOMContentLoaded", () => {
  setTimeout(checkUptimeForAllSites, 1000); // Wait a bit for theme to be applied
});
