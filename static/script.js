let clockInterval;

document.addEventListener("DOMContentLoaded", () => {
  setClock();
  initUptimeSSE();
});

function setClock() {
  let timeElement = document.getElementById("time");
  if (timeElement) {
    let data = timeElement.dataset.format;
    // Convert initial UTC time to local time if displayed
    convertInitialUTCToLocal(timeElement);
    updateTimeAndBackground(timeElement, data);
    clockInterval = setInterval(() => {
      updateTimeAndBackground(timeElement, data);
    }, 1000);
  }
}

// Function to convert initial UTC time to user's local time
function convertInitialUTCToLocal(timeElement) {
  // Get the current content of the time element (initially showing UTC time)
  const timeText = timeElement.textContent.trim();
  
  // Check if it's showing UTC time (has "UTC" suffix)
  if (timeText && timeText.includes(" UTC")) {
    // Parse the UTC time
    const utcTimeString = timeText.replace(" UTC", "").trim();
    
    // Create a date object from the time string
    // This assumes only the time (HH:MM:SS) is provided, so we'll use today's date
    const now = new Date();
    const [hours, minutes, seconds] = utcTimeString.split(':').map(Number);
    
    // Create a date object with UTC time
    const utcDate = new Date(Date.UTC(now.getFullYear(), now.getMonth(), now.getDate(), hours, minutes, seconds));
    
    // Convert to local time
    const localHours = utcDate.getHours();
    const localMinutes = utcDate.getMinutes().toString().padStart(2, '0');
    const localSeconds = utcDate.getSeconds().toString().padStart(2, '0');
    
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
    timeElement.textContent = localTimeString;
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
    let ampm = hours >= 12 ? "PM" : "AM";
    timeElement.innerText = `${displayHours}:${minutes} ${ampm}`;
  } else {
    timeElement.innerText = `${hours}:${minutes}`;
  }

  // Update background based on time of day
  updateTimeOfDayBackground(now);
}

function updateTimeOfDayBackground(date) {
  const body = document.body;
  const hour = date.getHours();

  if (hour >= 5 && hour < 12) {
    body.setAttribute("data-time", "morning");
  } else if (hour >= 12 && hour < 17) {
    body.setAttribute("data-time", "afternoon");
  } else if (hour >= 17 && hour < 21) {
    body.setAttribute("data-time", "evening");
  } else {
    body.setAttribute("data-time", "night");
  }
}

window.addEventListener("beforeunload", () => {
  clearInterval(clockInterval);
});

// Initialize SSE connection for uptime updates
function initUptimeSSE() {
  const eventSource = new EventSource("/uptime");

  eventSource.onmessage = function (event) {
    try {
      const uptimeData = JSON.parse(event.data);

      uptimeData.forEach(function (uptimeInfo) {
        // Find the site card that matches this site_id
        const siteCards = document.querySelectorAll(".site-card");
        siteCards.forEach(function (card) {
          const siteLink = card.querySelector("a span");
          const siteName = siteLink.textContent.trim();

          // Compare site name with the site_id from the SSE data
          if (siteName === uptimeInfo.site_id) {
            const uptimeElement = card.querySelector(".uptime");
            if (uptimeElement) {
              // Update the uptime element based on status
              uptimeElement.className = `uptime ${uptimeInfo.status}`;

              // Set the display text with status and uptime percentage
              let statusText =
                uptimeInfo.status.charAt(0).toUpperCase() +
                uptimeInfo.status.slice(1);
              const uptimePercentage = uptimeInfo.uptime_percentage.toFixed(1);

              // Add status indicator and percentage
              uptimeElement.innerHTML = `<span>●</span> ${statusText} (${uptimePercentage}%)`;
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
