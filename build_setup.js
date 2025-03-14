import fs from "fs";

let sourceDep = "";
const isWindows = process.platform === "win32";
switch (process.platform) {
	case "win32":
		sourceDep = "src-tauri/bin/win32-x64";
		break;
	case "darwin":
		sourceDep = "src-tauri/bin/macos-x64";
		break;
	case "linux":
		sourceDep = "src-tauri/bin/linux-x64";
		break;
}
let commonDep = "src-tauri/bin/common";

const targetDep = "src-tauri/bin/dependency";

fs.rmSync(targetDep, { recursive: true, force: true });
fs.cpSync(sourceDep, targetDep, { recursive: true });
fs.cpSync(commonDep, targetDep, { recursive: true });

console.log(`Copied ${sourceDep} to ${targetDep}`);
console.log(`Copied ${commonDep} to ${targetDep}`);