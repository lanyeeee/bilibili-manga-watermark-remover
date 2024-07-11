import {commands, JpgImageData} from "./bindings.ts";
import {Ref} from "vue";
import {path} from "@tauri-apps/api";
import {BaseDirectory, exists} from "@tauri-apps/plugin-fs";

export async function loadBackground(blackBackground: Ref<JpgImageData | undefined>, whiteBackground: Ref<JpgImageData | undefined>) {
    const load = async (isBlack: boolean) => {
        const filename = isBlack ? "black.png" : "white.png";
        const backgroundExist = await exists(filename, {baseDir: BaseDirectory.Resource});
        if (!backgroundExist) {
            return;
        }
        const backgroundPath = await get_background_path(isBlack);
        const result = await commands.openImage(backgroundPath);
        if (result.status === "error") {
            console.error(result.error);
            return;
        }
        const background = isBlack ? blackBackground : whiteBackground;
        background.value = result.data;
    };
    await Promise.all([load(true), load(false)]);
}

export async function get_background_path(isBlack: boolean) {
    const cacheDir = await path.resourceDir();
    const filename = isBlack ? "black.png" : "white.png";
    return path.join(cacheDir, filename);
}