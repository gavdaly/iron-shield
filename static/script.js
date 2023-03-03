let clockInterval;
(document.onDOMContentLoaded = () => {
    setClock()
})()

function setClock() {
    let clock = document.getElementById("clock");
    if (clock) {
        let data = clock.dataset.format
        clockInterval = setInterval(() => {
            let time = new Date();
            let hours = time.getHours()
            let minutes = ('0' + time.getMinutes()).slice(-2)
            if (data == "military") {
                clock.innerText = `${hours}:${minutes}`
            }
            else {
                let apm = (hours % 12 == 0) ? 'am' : 'pm'
                clock.innerText = `${(hours / 12).toFixed(0)}:${minutes} ${apm}`
            }
        }, 100)
    }
}

document.onclose(() => { clearInterval(clockInterval) })