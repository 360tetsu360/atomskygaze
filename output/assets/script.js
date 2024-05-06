const rows = 18;
const columns = 32;

const gridOverlay = document.getElementById("gridOverlay");
var is_clicking_left = false;
var is_clicking_right = false;

var grid_state = new Uint8Array(rows * columns);

for (let row = 0; row < rows; row++) {
    for (let col = 0; col < columns; col++) {
        const gridItem = document.createElement("div");
        gridItem.classList.add("grid-item");
        gridItem.id = `grid-${row}-${col}`;
        gridItem.addEventListener("mousedown", function(event) {
	if(event.which === 1) {
        this.style.backgroundColor = "rgba(255, 0, 0, 0.7)";
		is_clicking_left = true;
        grid_state[row * columns + col] = 1;
	}
	else if (event.which === 3) {
		this.style.backgroundColor = "rgba(0, 0, 0, 0.1)";
		is_clicking_right = true;
        grid_state[row * columns + col] = 0;
	}
    });
    gridItem.addEventListener("mouseover", function() {
	    if(is_clicking_left) {
            this.style.backgroundColor = "rgba(255, 0, 0, 0.7)";
            grid_state[row * columns + col] = 1;
	    }else if(is_clicking_right) {
		    this.style.backgroundColor = "rgba(0, 0, 0, 0.1)";
            grid_state[row * columns + col] = 0;
	    }
      });
      gridOverlay.appendChild(gridItem);
    }
}

document.body.addEventListener("mouseup", function(event) {
	if(event.which === 1) {
	    if(is_clicking_left) {
	        is_clicking_left = false;
	    }
	} else if(event.which === 3) {
	    if(is_clicking_right) {
	        is_clicking_right = false;	
	    }
	}
});

document.addEventListener("contextmenu", function(event) {
    event.preventDefault(); 
});

var atom_time = null;
const host = window.location.host;
var connection = new WebSocket(`ws://${host}/ws`);

connection.onopen = function(event) {
    console.log("Connected");
};

connection.onerror = function(error) {
    console.log(error);
};

var prev = document.getElementById("prev");
var blob = null;
connection.onmessage = function(event) {
    if (typeof event.data === 'string') {
        if (event.data === "") {
            return;
        }

        const packet = JSON.parse(event.data);

        if (packet.type === "time") {
            atom_time = packet.payload;
        } else if (packet.type === "detected") {
            const timestamp = packet.payload.timestamp;
            const record_path = packet.payload.saved_file;
            var log_box = document.getElementById("log-box");

            var new_item = document.createElement("div");
            new_item.className = "list-item";

            var oneline = document.createElement("div");
            oneline.className = "one-line";
            oneline.textContent = `[${timestamp}] Meteor Detected`;

            new_item.appendChild(oneline);
            new_item.onclick = () => {
                document.getElementById("video-dialog-title").textContent = `[${timestamp}] Meteor Detected`;
                document.getElementById("videoframe").src = `/view?filename=${record_path}`;
                document.getElementById("download").href = `/download?filename=${record_path}`;
                const dialog = document.getElementById("video-dialog");
                dialog.showModal();
            };

            if (log_box.firstChild) {
                log_box.insertBefore(new_item, log_box.firstChild);
            } else {
                log_box.appendChild(new_item);
            }
        } else if (packet.type === "appstate") {
            const app_state = packet.payload;

            grid_state = Uint8Array.from(app_state.mask);
            for (let row = 0; row < rows; row++) {
                for (let col = 0; col < columns; col++) {
                    if (grid_state[row * columns + col] === 1) {
                        let grid = document.getElementById(`grid-${row}-${col}`);
                        grid.style.backgroundColor = "rgba(255, 0, 0, 0.7)";
                    }
                }
            }
            document.getElementById("det").checked = app_state.detect;
            document.getElementById("tstmp").checked = app_state.timestamp;
            document.getElementById("night-mode").checked = app_state.night_mode;
            document.getElementById("ircut").checked = app_state.ircut_on;
            document.getElementById("led").checked = app_state.led_on;
            document.getElementById("irled").checked = app_state.irled_on;
            document.getElementById("hflip").checked = app_state.flip[0];
            document.getElementById("vflip").checked = app_state.flip[1];
            document.getElementById("fps-range").value = app_state.fps;
            document.getElementById("fps-value").textContent = `${app_state.fps}`;
            document.getElementById("brt-range").value = app_state.brightness;
            document.getElementById("brt-value").textContent = `${app_state.brightness}`;
            document.getElementById("cnt-range").value = app_state.contrast;
            document.getElementById("cnt-value").textContent = `${app_state.contrast}`;
            document.getElementById("shrp-range").value = app_state.sharpness;
            document.getElementById("shrp-value").textContent = `${app_state.sharpness}`;
            document.getElementById("sat-range").value = app_state.saturation;
            document.getElementById("sat-value").textContent = `${app_state.saturation}`;

            for (const log_item of app_state.logs) {
                console.log(log_item);
                const timestamp = log_item.Detection[0];
                const record_path = log_item.Detection[1];
                var log_box = document.getElementById("log-box");

                var new_item = document.createElement("div");
                new_item.className = "list-item";

                var oneline = document.createElement("div");
                oneline.className = "one-line";
                oneline.textContent = `[${timestamp}] Meteor Detected`;
                new_item.appendChild(oneline);
                new_item.onclick = () => {
                    document.getElementById("video-dialog-title").textContent = `[${timestamp}] Meteor Detected`;
                    document.getElementById("videoframe").src = `/view?filename=${record_path}`;
                    document.getElementById("download").href = `/download?filename=${record_path}`;
                    const dialog = document.getElementById("video-dialog");
                    dialog.showModal();
                };
    
                if (log_box.firstChild) {
                    log_box.insertBefore(new_item, log_box.firstChild);
                } else {
                    log_box.appendChild(new_item);
                }
            }

        }
      } else if (event.data instanceof Blob) {
        if (blob) {
            URL.revokeObjectURL(blob);
        }
        const binaryData = event.data;
        blob = new Blob([binaryData], { type: 'image/jpeg' });
        prev.src = URL.createObjectURL(blob);
      } else {
        console.log('Unknown message type');
      }

};

connection.onclose = function() {
    console.log("Close");
};

const yyyymmdd = new Intl.DateTimeFormat(
    undefined,
    {
      year:   'numeric',
      month:  '2-digit',
      day:    '2-digit',
      hour:   '2-digit',
      minute: '2-digit',
      second: '2-digit',
    }
  )

function update_time() {
    if(atom_time) {
        var cam_time = new Date(Number(atom_time.time) + 500);
        document.getElementById("atom-time").innerHTML = yyyymmdd.format(cam_time);
    }
    var dev_time = new Date();
    document.getElementById("dev-time").innerHTML = yyyymmdd.format(dev_time);
    connection.send("time");
}
setInterval('update_time()',500);

document.getElementById("sync").onclick = () => {
    var now = (new Date().valueOf() / 1000).toFixed(2);
    const secs = Math.floor(now);
    const millis = Math.floor((now - secs) * 1000);
    console.log(`sync,${secs},${millis}`);
    connection.send(`sync,${secs},${millis}`);
}

document.getElementById("wifi-settings").onclick = () => {
    const dialog = document.getElementById("wifi-dialog");
    dialog.showModal();
}

document.getElementById("wifi-dialog-close").onclick = () => {
    const dialog = document.getElementById("wifi-dialog");
    dialog.close();
}

document.getElementById("video-dialog-close").onclick = () => {
    const dialog = document.getElementById("video-dialog");
    dialog.close();
}

document.getElementById("shw_msk").onchange = () => {
    let checked = document.getElementById("shw_msk").checked; 
    var elements = document.getElementsByClassName("grid-item");
    for (var i = 0; i < elements.length; i++){
        elements[i].style.display = checked ? "block" : "none";
    }
}

document.getElementById("app-msk").onclick = () => {
    connection.send(grid_state.buffer);
}

document.getElementById("clear-msk").onclick = () => {
    for (let row = 0; row < rows; row++) {
        for (let col = 0; col < columns; col++) {
            if (grid_state[row * columns + col] === 1) {
                let grid = document.getElementById(`grid-${row}-${col}`);
                grid.style.backgroundColor = "rgba(0, 0, 0, 0.1)";
                grid_state[row * columns + col] = 0;
            }
        }
    }
}

document.getElementById("det").onchange= () => {
    let checked = document.getElementById("det").checked;
    connection.send(`det,${checked? "on":"off"}`);
}

document.getElementById("tstmp").onchange= () => {
    let checked = document.getElementById("tstmp").checked;
    connection.send(`tstmp,${checked? "on":"off"}`);
}

document.getElementById("save-conf").onclick = () => {
    connection.send("save");
}

document.getElementById("night-mode").onchange= () => {
    let checked = document.getElementById("night-mode").checked;
    connection.send(`mode,${checked? "night":"day"}`);
}

document.getElementById("ircut").onchange= () => {
    let checked = document.getElementById("ircut").checked;
    connection.send(`ir,${checked? "on":"off"}`);
}

document.getElementById("led").onchange= () => {
    let checked = document.getElementById("led").checked;
    connection.send(`led,${checked? "on":"off"}`);
}

document.getElementById("irled").onchange= () => {
    let checked = document.getElementById("irled").checked;
    connection.send(`irled,${checked? "on":"off"}`);
}

document.getElementById("hflip").onchange= () => {
    let checked = document.getElementById("hflip").checked;
    connection.send(`flip,h,${checked? "on":"off"}`);
}

document.getElementById("vflip").onchange= () => {
    let checked = document.getElementById("vflip").checked;
    connection.send(`flip,v,${checked? "on":"off"}`);
}

document.getElementById("fps").onclick = () => {
    connection.send(`fps,${document.getElementById("fps-range").value}`);
}

document.getElementById("brt").onclick= () => {
    connection.send(`proc,brt,${document.getElementById("brt-range").value}`);
}

document.getElementById("cnt").onclick= () => {
    connection.send(`proc,cnt,${document.getElementById("cnt-range").value}`);
}

document.getElementById("shrp").onclick= () => {
    connection.send(`proc,shrp,${document.getElementById("shrp-range").value}`);
}

document.getElementById("sat").onclick = () => {
    connection.send(`proc,sat,${document.getElementById("sat-range").value}`);
}

document.getElementById("reboot").onclick = () => {
    connection.send("reboot");
}

document.getElementById("det-settings").onclick = () => {
    const dialog = document.getElementById("detection-dialog");
    dialog.showModal();
}

document.getElementById("detection-dialog-close").onclick = () => {
    const dialog = document.getElementById("detection-dialog");
    dialog.close();
}

function isValidPsk(psk) {
    if (psk.length < 8 || psk.length > 63) {
        return false;
    }

    const asciiRegex = /^[\x20-\x7E]+$/;
    if (!asciiRegex.test(psk)) {
        return false;
    }

    return true;
}

let checkedap = document.getElementById("ap-mode").checked;
document.getElementById("ssid").disabled = !checkedap;
document.getElementById("psk").disabled = !checkedap;
document.getElementById("app-net").disabled = !checkedap;
document.getElementById("ap-mode").onchange= () => {
    let checked = document.getElementById("ap-mode").checked;
    document.getElementById("ssid").disabled = !checked;
    document.getElementById("psk").disabled = !checked;
    document.getElementById("app-net").disabled = !checked;
}

document.getElementById("app-net").onclick = () => {
    if (confirm("This connection is not secure. Do you still want to send it?\nこの接続は盗聴される恐れがあります。それでも送信しますか?\n\nヒント:より安全に設定する場合はSDカードのatom_config.tomlを変更してください。")) {
        let ap_mode = document.getElementById("ap-mode").checked;
        let ssid = document.getElementById("ssid").value;
        let psk = document.getElementById("psk").value;
        if (!isValidPsk(psk)) {
            alert("PSK must be 8 to 63 characters long and contain only printable ASCII characters.\nPSKは8文字から63文字の長さで、印刷可能なASCII文字のみを含む必要があります。");
        }
        connection.send(`net,${ap_mode},${ssid},${psk}`);
        alert("Changes will apply after rebooting.\n変更は再起動後、適用されます。")
    } else {

    }
}

const ranges = document.querySelectorAll('input[type="range"]');
const paragraphs = {
    "fps-range": document.getElementById("fps-value"),
    "brt-range": document.getElementById("brt-value"),
    "cnt-range": document.getElementById("cnt-value"),
    "shrp-range": document.getElementById("shrp-value"),
    "sat-range": document.getElementById("sat-value"),
};

function updateParagraph(event) {
    const sliderId = event.target.id;
    const newValue = event.target.value; 
    const paragraph = paragraphs[sliderId];
    if (paragraph) {
        paragraph.textContent = newValue;
    }
}

ranges.forEach(range => {
    range.addEventListener('input', updateParagraph);
});