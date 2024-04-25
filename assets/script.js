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

document.getElementById("det").onchange= () => {
    let checked = document.getElementById("det").checked;
    connection.send(`det,${checked? "on":"off"}`);
}

document.getElementById("tstmp").onchange= () => {
    let checked = document.getElementById("tstmp").checked;
    connection.send(`tstmp,${checked? "on":"off"}`);
}

document.getElementById("night-mode").onchange= () => {
    let checked = document.getElementById("night-mode").checked;
    connection.send(`mode,${checked? "night":"day"}`);
}

document.getElementById("ircut").onchange= () => {
    let checked = document.getElementById("ircut").checked;
    connection.send(`ir,${checked? "on":"off"}`);
}

document.getElementById("hflip").onchange= () => {
    let checked = document.getElementById("hflip").checked;
    connection.send(`flip,h,${checked? "on":"off"}`);
}

document.getElementById("vflip").onchange= () => {
    let checked = document.getElementById("vflip").checked;
    connection.send(`flip,v,${checked? "on":"off"}`);
}

document.getElementById("freeze-ae").onchange= () => {
    let checked = document.getElementById("freeze-ae").checked;
    connection.send(`ae,freeze,${checked? "on":"off"}`);
}

document.getElementById("expr-en").onchange= () => {
    let checked = document.getElementById("expr-en").checked;
    document.getElementById("expr-range").disabled = !checked;
    document.getElementById("expr").disabled = !checked;
    connection.send(`ae,expr-en,${checked? "on":"off"}`);
}

document.getElementById("expr").onclick= () => {
    connection.send(`ae,expr,${document.getElementById("expr-range").value}`);
}

document.getElementById("again-en").onchange= () => {
    let checked = document.getElementById("again-en").checked;
    document.getElementById("again-range").disabled = !checked;
    document.getElementById("again").disabled = !checked;
    connection.send(`ae,again-en,${checked? "on":"off"}`);
}

document.getElementById("again").onclick= () => {
    connection.send(`ae,again,${document.getElementById("again-range").value}`);
}

document.getElementById("dgain-en").onchange= () => {
    let checked = document.getElementById("dgain-en").checked;
    document.getElementById("dgain-range").disabled = !checked;
    document.getElementById("dgain").disabled = !checked;
    connection.send(`ae,dgain-en,${checked? "on":"off"}`);
}

document.getElementById("dgain").onclick= () => {
    connection.send(`ae,dgain,${document.getElementById("dgain-range").value}`);
}

document.getElementById("ispgain-en").onchange= () => {
    let checked = document.getElementById("ispgain-en").checked;
    document.getElementById("ispgain-range").disabled = !checked;
    document.getElementById("ispgain").disabled = !checked;
    connection.send(`ae,ispgain-en,${checked? "on":"off"}`);
}

document.getElementById("ispgain").onclick= () => {
    connection.send(`ae,ispgain,${document.getElementById("ispgain-range").value}`);
}

document.getElementById("wdr-en").onclick= () => {
    let checked = document.getElementById("wdr-en").checked;
    connection.send(`wdr,${checked? "on":"off"}`);
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

const ranges = document.querySelectorAll('input[type="range"]');
const paragraphs = {
    "expr-range": document.getElementById("expr-value"),
    "again-range": document.getElementById("again-value"),
    "dgain-range": document.getElementById("dgain-value"),
    "ispgain-range": document.getElementById("ispgain-value"),
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