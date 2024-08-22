const rows = 18;
const columns = 32;
// Time zone picker from
const tzInts = [
    {"label":"(UTC-12:00)", "value":-43200},
    {"label":"(UTC-11:00)", "value":-39600},
    {"label":"(UTC-10:00)", "value":-36000},
    {"label":"(UTC-09:00)", "value":-32400},
    {"label":"(UTC-08:00)", "value":-28800},
    {"label":"(UTC-07:00)", "value":-25200},
    {"label":"(UTC-06:00)", "value":-21600},
    {"label":"(UTC-05:00)", "value":-18000},
    {"label":"(UTC-04:00)", "value":-14400},
    {"label":"(UTC-03:00)", "value":-10800},
    {"label":"(UTC-02:00)", "value":-7200},
    {"label":"(UTC-01:00)", "value":-3600},
    {"label":"(UTC+00:00)", "value":0},
    {"label":"(UTC+01:00)", "value":3600},
    {"label":"(UTC+02:00)", "value":7200},
    {"label":"(UTC+03:00)", "value":10800},
    {"label":"(UTC+03:30)", "value":12600},
    {"label":"(UTC+04:00)", "value":14400},
    {"label":"(UTC+04:30)", "value":16200},
    {"label":"(UTC+05:00)", "value":18000},
    {"label":"(UTC+05:30)", "value":19800},
    {"label":"(UTC+05:45)", "value":20700},
    {"label":"(UTC+06:00)", "value":21600},
    {"label":"(UTC+06:30)", "value":23400},
    {"label":"(UTC+07:00)", "value":25200},
    {"label":"(UTC+08:00)", "value":28800},
    {"label":"(UTC+08:45)", "value":31500},
    {"label":"(UTC+09:00)", "value":32400},
    {"label":"(UTC+09:30)", "value":34200},
    {"label":"(UTC+10:00)", "value":36000},
    {"label":"(UTC+10:30)", "value":37800},
    {"label":"(UTC+11:00)", "value":39600},
    {"label":"(UTC+12:00)", "value":43200},
    {"label":"(UTC+12:45)", "value":45900},
    {"label":"(UTC+13:00)", "value":46800},
    {"label":"(UTC+14:00)", "value":50400}
];

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

document.getElementById("timezone").appendChild(timezoneSelect());

var atom_time = null;
const host = window.location.host;
var connection = new WebSocket(`ws://${host}/ws`);

connection.onopen = function(event) {
    console.log("Connected");
};

connection.onerror = function(error) {
    console.log(error);
};

function format_time(time) {
    let h_str = time[0].toString().padStart( 2, '0');
    let m_str = time[1].toString().padStart( 2, '0');
    return `${h_str}:${m_str}`
}

var prev = document.getElementById("prev");
var prev_dialog = document.getElementById("prev-dialog-prev");
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
                document.getElementById("imageframe").src = `/view?filename=${record_path}`;
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
            document.getElementById("det-mode").value = app_state.detection_config.detection_time != null;
            if (app_state.detection_config.detection_time != null) {
                let start_time = app_state.detection_config.detection_time.start;
                let end_time = app_state.detection_config.detection_time.end;
                document.getElementById("start-time").value = format_time(start_time);
                document.getElementById("end-time").value = format_time(end_time);
            }
            let show_time = document.getElementById("det-mode").checked;
            document.getElementById("det-time-panel").style.display = show_time ? "" : "none";
            document.getElementById("det-time-title").style.display = show_time ? "none" : "";
            document.getElementById("det-mode-sub").checked = show_time;

            console.log(app_state.detection_config);
            document.getElementById("det-ana").checked = app_state.detection_config.solve_field;
            document.getElementById("wcs").checked = app_state.detection_config.save_wcs;
            document.getElementById("const").checked = app_state.detection_config.draw_constellation;
            let show = document.getElementById("det-ana").checked;
            document.getElementById("det-ana-panel").style.display = show ? "" : "none";
            document.getElementById("det-ana-title").style.display = show ? "none" : "";
            document.getElementById("det-ana-sub").checked = show;
            let offset = app_state.timezone;
            var select = document.getElementById("tzselect");
            for (var j = 0; j < select.options.length; j++) {
                if (select.options[j].value == offset) {
                    select.options[j].selected = true;
                    break;
                }
            }

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
        let url = URL.createObjectURL(blob);
        prev.src = url;
        prev_dialog.src = url;
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
    connection.send(`sync,${secs},${millis}`);
}

document.getElementById("tzselect").onchange = () => {
    var timezone = document.getElementById("tzselect").value;
    connection.send(`tz,${timezone}`);
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

document.getElementById("capture").onclick = () => {
    connection.send("cap");
}

document.getElementById("det-settings").onclick = () => {
    const dialog = document.getElementById("detection-dialog");
    dialog.showModal();
}

document.getElementById("detection-dialog-close").onclick = () => {
    const dialog = document.getElementById("detection-dialog");
    dialog.close();
}

document.getElementById("prev-dialog-close").onclick = () => {
    const dialog = document.getElementById("prev-dialog");
    dialog.close();
}

document.getElementById("zoom").onclick = () => {
    const dialog = document.getElementById("prev-dialog");
    dialog.showModal();
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
    if (confirm("This connection is not secure. Do you still want to send it?\nこの接続は保護されていません。それでも送信しますか?\n\nヒント:より安全に設定する場合はSDカードのatom_config.tomlを変更してください。")) {
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

document.getElementById("det-mode").onchange= () => {
    let start = document.getElementById("start-time").value;
    let end = document.getElementById("end-time").value;
    let use_time = document.getElementById("det-mode").checked;
    document.getElementById("det-time-panel").style.display = use_time ? "" : "none";
    document.getElementById("det-time-title").style.display = use_time ? "none" : "";
    document.getElementById("det-mode-sub").checked = use_time;
    connection.send(`det-time,${use_time},${start == "" ? "null" : start},${end == "" ? "null" : end}`);
}

document.getElementById("det-ana").onchange= () => {
    let solve = document.getElementById("det-ana").checked;
    let wcs = document.getElementById("wcs").checked;
    let constellation = document.getElementById("const").checked;
    document.getElementById("det-ana-panel").style.display = solve ? "" : "none";
    document.getElementById("det-ana-title").style.display = solve ? "none" : "";
    document.getElementById("det-ana-sub").checked = solve;
    console.log(`solve,${solve},${wcs},${constellation}`)
    connection.send(`solve,${solve},${wcs},${constellation}`);
}

document.getElementById("app-det").onclick = () => {
    let start = document.getElementById("start-time").value;
    let end = document.getElementById("end-time").value;
    let use_time = document.getElementById("det-mode").checked;
    connection.send(`det-time,${use_time},${start == "" ? "null" : start},${end == "" ? "null" : end}`);
}

document.getElementById("app-det-ana").onclick = () => {
    let solve = document.getElementById("det-ana").checked;
    let wcs = document.getElementById("wcs").checked;
    let constellation = document.getElementById("const").checked;
    console.log(`solve,${solve},${wcs},${constellation}`)
    connection.send(`solve,${solve},${wcs},${constellation}`);
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

function timezoneSelect(){
    var select = document.createElement("select");
    select.id = "tzselect";

    for (var i=0; i<tzInts.length; i++){
      var tz = tzInts[i],
          option = document.createElement("option");

      option.value = tz.value
      option.appendChild(document.createTextNode(tz.label))
      select.appendChild(option)
    }

    return select;
}