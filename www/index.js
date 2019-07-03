import * as wasm from "wasm-webgl";

const playPauseButton = document.getElementById("play-pause");

const play = () => {
    playPauseButton.textContent = "▶";
};

playPauseButton.addEventListener("click", event => {
    playPauseButton.disabled = true;
    wasm.start();
});

play();
