let clockInterval;
(document.onDOMContentLoaded = () => {
    setClock()
})()

function setClock() {
    let clock = document.getElementById("clock");
    if (clock) {
        clockInterval = setInterval(()=> {
            let time = new Date();
            clock.innerText = `${time.getHours()}:${time.getMinutes()}`
        }, 100)
    }
}

document.onclose(() => {clearInterval(clockInterval)})