// Apply the one-off load-time transforms via classes (animated by .animate-icon)
const earth = document.getElementById("icon-earth");
earth.classList.add("rotated");

const poison = document.getElementById("icon-poison");
poison.classList.add("scaled");

// Toggle a darker colour on the bus icon when clicked
document.getElementById("icon-bus").addEventListener("click", function (event) {
    event.target.classList.toggle("dark-color");
});
