export const messageChannelHeaderLayout = {
  leftColumn: 'min-w-0 flex-1',
  leftMeta: 'min-w-0',
  controlsRow: 'mt-3 flex flex-wrap items-center gap-2',
  accountSelect: 'oc-select oc-channel-account-select h-8 min-w-[220px] text-sm',
  accountSelectTrigger:
    'w-[232px] max-w-full min-w-[208px] !h-8 px-3 text-left justify-between',
  accountSelectMenu: 'oc-dropdown-menu absolute z-20 mt-1 w-full max-h-56 overflow-auto',
  accountButton: 'h-8 whitespace-nowrap',
  rightAction: 'shrink-0 whitespace-nowrap',
  toggleWrap: 'oc-channel-toggle-wrap shrink-0',
  toggleControl: 'oc-channel-toggle',
  toggleThumb: 'oc-channel-toggle-thumb',
  icon: 'shrink-0',
} as const
