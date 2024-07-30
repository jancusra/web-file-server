var earth = document.getElementById("icon-earth");
earth.style.transform = "rotate(1800deg)";

var poison = document.getElementById("icon-poison");
poison.style.transform = "scale(2)";

document.getElementById("icon-bus").addEventListener("click", function (e) {
    e.target.classList.toggle("dark-color");
}, false);
