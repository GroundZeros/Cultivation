import { invoke } from '@tauri-apps/api/core'
import { basename, dirname, join } from '@tauri-apps/api/path'
import { getConfigOption } from './configuration'

export const getGrasscutterJar = () => getConfigOption('grasscutter_path')

export async function getGameExecutable() {
  const exe_path = await getConfigOption('game_install_path')
  return await basename(exe_path)
}

export async function getGameVersion() {
  const execPath = await getConfigOption('game_install_path')
  const rootPath = await dirname(execPath)
  const baseName = await basename(execPath, '.exe')
  const datapath = await join(rootPath, `${baseName}_Data`)
  const asbPath = await join(datapath, 'StreamingAssets', 'asb_settings.json')
  const hasAsb = await invoke<boolean>('dir_exists', { path: asbPath })

  if (!hasAsb) {
    const versionFile = await join(datapath, 'StreamingAssets', 'BinaryVersion.bytes')
    const rawVersion = await invoke<string>('read_file', { path: versionFile })
    if (!rawVersion) return null

    const [major, minor] = rawVersion.split('.').map(Number)
    return { major, minor, release: 0 }
  }

  const settings = JSON.parse(await invoke<string>('read_file', { path: asbPath }))
  const [major, minorRelease] = settings.variance.split('.')
  const [minor, release] = minorRelease.split('_').map(Number)

  return { major: parseInt(major), minor, release }
}
