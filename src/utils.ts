import { commands } from './bindings.ts'
import { NotificationApiInjection } from 'naive-ui/es/notification/src/NotificationProvider'

export async function showPathInFileManager(path: string | undefined) {
  if (path === undefined) {
    return
  }
  await commands.showPathInFileManager(path)
}

export async function getBackgroundDirRelativePath(
  mangaDir: string,
  width: number,
  height: number,
  notification: NotificationApiInjection,
): Promise<string | null> {
  const result = await commands.getBackgroundDirRelativePath(mangaDir, width, height)
  if (result.status === 'error') {
    notification.error({ title: '获取背景水印图相对路径失败', description: result.error })
    return null
  }
  return result.data
}

export async function getBackgroundDirAbsPath(
  mangaDir: string,
  width: number,
  height: number,
  notification: NotificationApiInjection,
): Promise<string | null> {
  const result = await commands.getBackgroundDirAbsPath(mangaDir, width, height)
  if (result.status === 'error') {
    notification.error({ title: '获取背景水印图绝对路径失败', description: result.error })
    return null
  }
  return result.data
}

export async function autoGenerateBackground(
  mangaDir: string,
  width: number,
  height: number,
  notification: NotificationApiInjection,
): Promise<boolean> {
  const result = await commands.generateBackground(mangaDir, null, width, height)
  if (result.status === 'error') {
    notification.error({
      title: `自动生成背景水印图(${width}x${height})失败`,
      description: result.error,
    })
    return false
  }
  return true
}
