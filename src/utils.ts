import {commands, JpgImageData} from "./bindings.ts";
import {Ref} from "vue";

export async function loadBackground(blackBackground: Ref<JpgImageData | undefined>, whiteBackground: Ref<JpgImageData | undefined>) {
    const load = async (isBlack: boolean) => {
        const existsResult = await commands.backgroundExists(isBlack);
        if (existsResult.status === "error") {
            console.error(existsResult.error);
            return;
        }
        const exists = existsResult.data;
        if (!exists) {
            return;
        }
        const openResult = await commands.openBackground(isBlack);
        if (openResult.status === "error") {
            console.error(openResult.error);
            return;
        }
        const background = isBlack ? blackBackground : whiteBackground;
        background.value = openResult.data;
    };
    await Promise.all([load(true), load(false)]);
}