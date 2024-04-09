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

var connection = new WebSocket("/ws");

connection.onopen = function(event) {
    console.log("Connected");
};

connection.onerror = function(error) {
    console.log(error);
};

connection.onmessage = function(event) {
    const binaryData = event.data;

    const blob = new Blob([binaryData], { type: 'image/jpeg' });

    document.getElementById("prev").src = URL.createObjectURL(blob);
};

connection.onclose = function() {
    console.log("Close");
};

var on = false;
document.getElementById("ircut").onclick = () => {
    if(on) {
        connection.send("iron");
    }else {
        connection.send("iroff");
    }
    on = !on;
}


document.getElementById("log").onclick = () => {
    connection.send("log");
}

var flip = 0;
document.getElementById("flip").onclick = () => {
    connection.send(`flip${flip}`);
    flip++;
    if(flip == 5) {
        flip = 0;
    }
}

var gain_type = "again";
document.getElementById("gain").onclick = () => {
    let vgain_cmd = `gain,${gain_type},${document.getElementById("vgain").value}`;
    console.log(vgain_cmd);
    connection.send(vgain_cmd);
}
document.getElementById("gaintyp").onclick = () => {
    if(gain_type == "again") {
        gain_type = "dgain"
    } else if(gain_type == "dgain") {
        gain_type = "again"
    }

    document.getElementById("gaintyp").innerHTML = gain_type;
}

var proc_type = "brt";
document.getElementById("proc").onclick = () => {
    let vproc_cmd = `proc,${proc_type},${document.getElementById("vproc").value}`;
    console.log(vproc_cmd);
    connection.send(vproc_cmd);
}
document.getElementById("proctyp").onclick = () => {
    if(proc_type == "brt") {
        proc_type = "cont"
    } else if(proc_type == "cont") {
        proc_type = "shrp"
    } else if(proc_type == "shrp") {
        proc_type = "satu"
    } else if(proc_type == "satu") {
        proc_type = "brt"
    }

    document.getElementById("proctyp").innerHTML = proc_type;
}

document.getElementById("expr").onclick = () => {
    let vexpr_cmd = `expr,${document.getElementById("vexpr").value}`;
    console.log(vexpr_cmd);
    connection.send(vexpr_cmd);
}

document.getElementById("whba").onclick = () => {
    let vwb_cmd = `expr,${document.getElementById("vwbr").value},${document.getElementById("vwbb").value}`;
    console.log(vwb_cmd);
    connection.send(vwb_cmd);
}


var mask_show = false;
document.getElementById("shw_msk").onclick = () => {
    if(mask_show) {
        document.getElementById("shw_msk").innerHTML = "hide mask";
        var elements = document.getElementsByClassName("grid-item");

        for (var i = 0; i < elements.length; i++){
            elements[i].style.display = "none";
        }
    } else {
        document.getElementById("shw_msk").innerHTML = "show mask";
        var elements = document.getElementsByClassName("grid-item");

        for (var i = 0; i < elements.length; i++){
            elements[i].style.display = "block";
        }
    }

    mask_show = !mask_show;
}

document.getElementById("app_msk").onclick = () => {
    connection.send(grid_state.buffer);
}
