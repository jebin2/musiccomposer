const invoke = window.__TAURI__.core.invoke;
const listen = window.__TAURI__.event.listen;

// --- Utility Functions ---
async function invokeAPI(method, ...args) {
	try {
		return await invoke(method, ...args);
	} catch (error) {
		console.error(`Error invoking ${method}:`, error);
		alert(`${error}`);
		setGeneratingState(false);
		throw error;
	}
}

const consoleElement = document.getElementById("console");

play_svg = `<svg width="64px" height="64px" viewBox="-0.5 0 7 7" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" fill="#000000"><g id="SVGRepo_bgCarrier" stroke-width="0"></g><g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"></g><g id="SVGRepo_iconCarrier"> <title>play [#1003]</title> <desc>Created with Sketch.</desc> <defs> </defs> <g id="Page-1" stroke="none" stroke-width="1" fill="none" fill-rule="evenodd"> <g id="Dribbble-Light-Preview" transform="translate(-347.000000, -3766.000000)" fill="#000000"> <g id="icons" transform="translate(56.000000, 160.000000)"> <path d="M296.494737,3608.57322 L292.500752,3606.14219 C291.83208,3605.73542 291,3606.25002 291,3607.06891 L291,3611.93095 C291,3612.7509 291.83208,3613.26444 292.500752,3612.85767 L296.494737,3610.42771 C297.168421,3610.01774 297.168421,3608.98319 296.494737,3608.57322" id="play-[#1003]"> </path> </g> </g> </g> </g></svg>`;
stop_svg = `<svg width="64px" height="64px" viewBox="-1 0 8 8" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" fill="#000000"><g id="SVGRepo_bgCarrier" stroke-width="0"></g><g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"></g><g id="SVGRepo_iconCarrier"> <title>pause [#1006]</title> <desc>Created with Sketch.</desc> <defs> </defs> <g id="Page-1" stroke="none" stroke-width="1" fill="none" fill-rule="evenodd"> <g id="Dribbble-Light-Preview" transform="translate(-227.000000, -3765.000000)" fill="#000000"> <g id="icons" transform="translate(56.000000, 160.000000)"> <path d="M172,3605 C171.448,3605 171,3605.448 171,3606 L171,3612 C171,3612.552 171.448,3613 172,3613 C172.552,3613 173,3612.552 173,3612 L173,3606 C173,3605.448 172.552,3605 172,3605 M177,3606 L177,3612 C177,3612.552 176.552,3613 176,3613 C175.448,3613 175,3612.552 175,3612 L175,3606 C175,3605.448 175.448,3605 176,3605 C176.552,3605 177,3605.448 177,3606" id="pause-[#1006]"> </path> </g> </g> </g> </g></svg>`

const playButton = document.querySelector('.play-button');
	const waveBars = document.querySelectorAll('.wave-bar');
	let isPlaying = false;

	playButton.addEventListener('click', function () {
		isPlaying = !isPlaying;

		if (isPlaying) {
			// Change button text/icon to indicate pause state
			playButton.innerHTML = stop_svg;
			playButton.classList.toggle("removeleft");

			// Animate wave bars
			waveBars.forEach(bar => {
				// Generate random animation duration between 0.4 and 1.2 seconds
				const animationDuration = (Math.random() * 0.8 + 0.4).toFixed(2);

				// Apply animation
				bar.style.animation = `waveAnimation ${animationDuration}s ease-in-out infinite alternate`;
				// bar.style.backgroundColor = 'var(--accent-color)';
			});

			// Define the animation in a style element
			if (!document.getElementById('waveAnimationStyle')) {
				const styleElement = document.createElement('style');
				styleElement.id = 'waveAnimationStyle';
				styleElement.innerHTML = `
                    @keyframes waveAnimation {
                        0% {
                            height: 10%;
                        }
                        25% {
                            height: ${Math.floor(Math.random() * 30) + 30}%;
                        }
                        50% {
                            height: ${Math.floor(Math.random() * 30) + 60}%;
                        }
                        75% {
                            height: ${Math.floor(Math.random() * 30) + 90}%;
                        }
                        100% {
                            height: ${Math.floor(Math.random() * 40) + 110}%;
                        }
                    }
                `;
				document.head.appendChild(styleElement);
			}
		} else {
			// Change button back to play icon
			playButton.innerHTML = play_svg;
			playButton.classList.toggle("removeleft");

			// Stop animation
			waveBars.forEach(bar => {
				bar.style.animation = 'none';
				bar.style.backgroundColor = 'var(--accent-secondary)';

				// Reset to original random heights
				// setTimeout(() => {
				//     const randomHeight = Math.floor(Math.random() * 60) + 30;
				//     bar.style.height = `${randomHeight}px`;
				// }, 100);
			});
		}
	});

	// Initialize play button with play icon
	playButton.innerHTML = play_svg;

function appendConsoleMessage(message) {
	consoleElement.innerHTML += `<p>${message}</p>`;
	autoScrollConsole();
}

function autoScrollConsole() {
	consoleElement.scrollTop = consoleElement.scrollHeight;
}

// --- Event Listeners for Tauri Events ---
listen('info', (event) => {
	appendConsoleMessage(event.payload);
});
listen('error', (event) => {
	appendConsoleMessage(`<span style="color:red">${event.payload}</span>`);
});
listen('success', (event) => {
	appendConsoleMessage(`<span style="color:green">${event.payload}</span>`);
});
document.getElementById("convert-button").addEventListener("click", async () => {
	await invokeAPI("install_fluidsynth");
});

function openURL(url) {
	window.__TAURI__.opener.openUrl(url);
}