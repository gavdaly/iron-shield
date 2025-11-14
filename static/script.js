let clockInterval;

document.addEventListener("DOMContentLoaded", () => {
  console.log("DOM fully loaded and parsed");
  setClock();
  initUptimeSSE();
});

function setClock() {
  console.log("setClock function called");
  let timeElement = document.getElementById("time");
  if (timeElement) {
    let data = timeElement.dataset.format;
    console.log("Time element found, format:", data);
    // Convert initial UTC time to local time if displayed
    convertInitialUTCToLocal(timeElement);
    updateTimeAndBackground(timeElement, data);
    clockInterval = setInterval(() => {
      console.log("Updating time and background");
      updateTimeAndBackground(timeElement, data);
    }, 1000);
  } else {
    console.log("Time element not found");
  }
}

// Function to convert initial UTC time to user's local time
function convertInitialUTCToLocal(timeElement) {
  console.log("convertInitialUTCToLocal function called");
  // Get the current content of the time element (initially showing UTC time)
  const timeText = timeElement.textContent.trim();
  console.log("Current time text:", timeText);

  // Check if it's showing UTC time (has "UTC" suffix)
  if (timeText && timeText.includes(" UTC")) {
    console.log("Converting UTC time to local time");
    // Parse the UTC time
    const utcTimeString = timeText.replace(" UTC", "").trim();
    console.log("Parsed UTC time string:", utcTimeString);

    // Create a date object from the time string
    // This assumes only the time (HH:MM:SS) is provided, so we'll use today's date
    const now = new Date();
    const [hours, minutes, seconds] = utcTimeString.split(":").map(Number);
    console.log("Parsed time components:", { hours, minutes, seconds });

    // Create a date object with UTC time
    const utcDate = new Date(
      Date.UTC(
        now.getFullYear(),
        now.getMonth(),
        now.getDate(),
        hours,
        minutes,
        seconds,
      ),
    );

    // Convert to local time
    const localHours = utcDate.getHours();
    const localMinutes = utcDate.getMinutes().toString().padStart(2, "0");
    const localSeconds = utcDate.getSeconds().toString().padStart(2, "0");
    console.log("Local time components:", {
      localHours,
      localMinutes,
      localSeconds,
    });

    // Format based on the clock format
    const format = timeElement.dataset.format;
    let localTimeString;

    if (format === "12hour") {
      // Convert to 12-hour format
      let displayHours = localHours % 12;
      if (displayHours === 0) displayHours = 12;
      const ampm = localHours >= 12 ? "PM" : "AM";
      localTimeString = `${displayHours}:${localMinutes}:${localSeconds} ${ampm}`;
    } else {
      // 24-hour format
      localTimeString = `${localHours}:${localMinutes}:${localSeconds}`;
    }

    // Update the time element with local time
    console.log("Setting local time text:", localTimeString);
    timeElement.textContent = localTimeString;
  } else {
    console.log("Time is not in UTC format, no conversion needed");
  }
}

function updateTimeAndBackground(timeElement, data) {
  console.log("updateTimeAndBackground function called with data:", data);
  let now = new Date();
  let hours = now.getHours();
  let minutes = ("0" + now.getMinutes()).slice(-2);
  let seconds = ("0" + now.getSeconds()).slice(-2);

  console.log("Current time components:", { hours, minutes, seconds });

  // Update time display
  if (data === "12hour") {
    let displayHours = hours % 12;
    if (displayHours === 0) displayHours = 12;
    let ampm = hours >= 12 ? "PM" : "AM";
    const timeString = `${displayHours}:${minutes} ${amppm}`;
    console.log("Setting 12-hour time:", timeString);
    timeElement.innerText = timeString;
  } else {
    const timeString = `${hours}:${minutes}`;
    console.log("Setting 24-hour time:", timeString);
    timeElement.innerText = timeString;
  }

  // Update background based on time of day
  updateTimeOfDayBackground(now);
}

function updateTimeOfDayBackground(date) {
  console.log("updateTimeOfDayBackground function called with date:", date);
  const body = document.body;
  const hour = date.getHours();
  console.log("Current hour:", hour);

  if (hour >= 5 && hour < 12) {
    console.log("Setting background to morning");
    body.setAttribute("data-time", "morning");
  } else if (hour >= 12 && hour < 17) {
    console.log("Setting background to afternoon");
    body.setAttribute("data-time", "afternoon");
  } else if (hour >= 17 && hour < 21) {
    console.log("Setting background to evening");
    body.setAttribute("data-time", "evening");
  } else {
    console.log("Setting background to night");
    body.setAttribute("data-time", "night");
  }
}

window.addEventListener("beforeunload", () => {
  console.log("Page is unloading, clearing clock interval");
  clearInterval(clockInterval);
});

// Initialize SSE connection for uptime updates
function initUptimeSSE() {
  console.log("Initializing uptime SSE connection");
  const eventSource = new EventSource("/uptime");

  eventSource.onmessage = function (event) {
    console.log("Received uptime message:", event.data);
    try {
      const uptimeData = JSON.parse(event.data);
      console.log("Parsed uptime data:", uptimeData);

      uptimeData.forEach(function (uptimeInfo) {
        console.log("Processing uptime info for site:", uptimeInfo.site_id);
        // Find the site card that matches this site_id
        const siteCards = document.querySelectorAll(".site-card");
        console.log("Found", siteCards.length, "site cards");
        siteCards.forEach(function (card) {
          const siteLink = card.querySelector("a span");
          if (siteLink) {
            const siteName = siteLink.textContent.trim();
            console.log("Checking site card with name:", siteName);

            // Compare site name with the site_id from the SSE data
            if (siteName === uptimeInfo.site_id) {
              console.log("Found matching site card for", uptimeInfo.site_id);
              const uptimeElement = card.querySelector(".uptime");
              if (uptimeElement) {
                console.log(
                  "Updating uptime element status:",
                  uptimeInfo.status,
                );
                // Update the uptime element based on status
                uptimeElement.className = `uptime ${uptimeInfo.status}`;

                // Set the display text with status and uptime percentage
                let statusText =
                  uptimeInfo.status.charAt(0).toUpperCase() +
                  uptimeInfo.status.slice(1);
                const uptimePercentage =
                  uptimeInfo.uptime_percentage.toFixed(1);

                // Add status indicator and percentage
                uptimeElement.innerHTML = `<span class="status-text">${statusText}</span><span class="uptime-percentage">${uptimePercentage}%</span>`;
                console.log(
                  "Updated uptime element HTML:",
                  uptimeElement.innerHTML,
                );
              } else {
                console.warn(
                  "Uptime element not found in card for",
                  uptimeInfo.site_id,
                );
              }
            }
          }
        });
      });
    } catch (error) {
      console.error("Error processing uptime data:", error);
    }
  };

  eventSource.onerror = function (event) {
    console.error("SSE connection error for uptime:", event);
    eventSource.close();
  };
}
