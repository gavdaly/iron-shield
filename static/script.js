let clockInterval;
(document.onDOMContentLoaded = () => {
  setClock();
})();

function setClock() {
  let clock = document.getElementById("clock");
  if (clock) {
    let data = clock.dataset.format;
    clockInterval = setInterval(() => {
      let time = new Date();
      let hours = time.getHours();
      let minutes = ("0" + time.getMinutes()).slice(-2);
      if (data === "12hour") {
        let displayHours = hours % 12;
        if (displayHours === 0) displayHours = 12;
        clock.innerText = `${displayHours}:${minutes} ${hours >= 12 ? "PM" : "AM"}`;
      } else {
        clock.innerText = `${hours}:${minutes}`;
      }
    }, 1000);
  }
}

window.addEventListener("beforeunload", () => {
  clearInterval(clockInterval);
});
