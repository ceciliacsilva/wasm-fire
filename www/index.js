import * as wasm from "wasm-webgl";

// wasm.say_hi();

const playPauseButton = document.getElementById("play-pause");

const play = () => {
    playPauseButton.textContent = "⏸";
    wasm.start();
};

const pause = () => {
    playPauseButton.textContent = "▶";
};

play();
