export interface AutoScrollStateInput {
  scrollTop: number
  clientHeight: number
  scrollHeight: number
  wasAutoScrollEnabled: boolean
}

const AUTO_SCROLL_BOTTOM_THRESHOLD_PX = 16

const isNearBottom = (input: AutoScrollStateInput) =>
  input.scrollHeight - (input.scrollTop + input.clientHeight) <= AUTO_SCROLL_BOTTOM_THRESHOLD_PX

export const shouldKeepAutoScroll = (input: AutoScrollStateInput) => {
  if (input.scrollHeight <= input.clientHeight) return true
  if (isNearBottom(input)) return true
  return false
}

