let clockInterval;
document.addEventListener("DOMContentLoaded", () => {
  setClock();
});

function setClock() {
  let timeElement = document.getElementById("time");
  if (timeElement) {
    let data = timeElement.dataset.format;
    clockInterval = setInterval(() => {
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
    }, 1000);
  }
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

window.addEventListener("beforeunload", () => {
  clearInterval(clockInterval);
});
