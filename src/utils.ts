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
        const cacheDir = await path.resourceDir();
        const backgroundPath = await path.join(cacheDir, filename);
        const result = await commands.openImage(backgroundPath);
        if (result.status === "error") {
            console.error(result.error);
            return;
        }
        const response = result.data;
        if (response.code !== 0) {
            console.error(response.msg);
            return;
        }
        const background = isBlack ? blackBackground : whiteBackground;
        background.value = response.data;
    };
    await Promise.all([load(true), load(false)]);
}