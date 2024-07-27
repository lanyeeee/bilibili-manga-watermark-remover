import {commands} from "./bindings.ts";
import {NotificationApiInjection} from "naive-ui/es/notification/src/NotificationProvider";

export async function showPathInFileManager(path: string | undefined) {
    if (path === undefined) {
        return;
    }
    await commands.showPathInFileManager(path);
}

export async function getBackgroundDirRelativePath(mangaDir: string, width: number, height: number, notification: NotificationApiInjection): Promise<string | null> {
    const dirResult = await commands.getBackgroundDirRelativePath(mangaDir, width, height);
    if (dirResult.status === "error") {
        notification.error({title: "获取背景水印图相对路径失败", description: dirResult.error});
        return null;
    }
    const dirResponse = dirResult.data;
    if (dirResponse.code !== 0) {
        notification.warning({title: "获取背景水印图相对路径失败", description: dirResponse.msg});
        return null;
    }
    return dirResponse.data;
}

export async function getBackgroundDirAbsPath(mangaDir: string, width: number, height: number, notification: NotificationApiInjection): Promise<string | null> {
    const dirResult = await commands.getBackgroundDirAbsPath(mangaDir, width, height);
    if (dirResult.status === "error") {
        notification.error({title: "获取背景水印图绝对路径失败", description: dirResult.error});
        return null;
    }
    const dirResponse = dirResult.data;
    if (dirResponse.code !== 0) {
        notification.warning({title: "获取背景水印图绝对路径失败", description: dirResponse.msg});
        return null;
    }
    return dirResponse.data;
}

export async function autoGenerateBackground(mangaDir: string, width: number, height: number, notification: NotificationApiInjection): Promise<boolean> {
    const generateResult = await commands.generateBackground(mangaDir, null, width, height);
    if (generateResult.status === "error") {
        notification.error({
            title: `自动生成背景水印图(${width}x${height})失败`,
            description: generateResult.error
        });
        return false;
    }
    const response = generateResult.data;
    if (response.code !== 0) {
        notification.warning({
            title: `自动生成背景水印图(${width}x${height})失败`,
            description: response.msg,
            content: "请尝试手动截取水印",
        });
        return false;
    }
    return true;
}