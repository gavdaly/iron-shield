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
            if (data == "military") {
                clock.innerText = `${time.getHours()}:${time.getMinutes()}`
            }
            else {
                let hours = time.getHours()
                let apm = (hours % 12 == 0) ? 'am' : 'pm'
                clock.innerText = `${Math.floor(hours / 12)}:${time.getMinutes()} ${apm}`
            }
        }, 100)
    }
}

document.onclose(() => { clearInterval(clockInterval) })