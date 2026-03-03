export function getChannelTestModeLabel(realSupported: boolean): string {
  return realSupported ? '真实送达测试' : '模拟测试'
}
