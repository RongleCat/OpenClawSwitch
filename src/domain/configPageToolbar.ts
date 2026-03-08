export const CONFIG_PAGE_DESCRIPTION = '选择配置文件、调整模型并保存。'

export const resolveConfigPagePrimaryActionState = ({
  canSave,
  loading,
  primaryModelInvalid,
}: {
  canSave: boolean
  loading: boolean
  primaryModelInvalid: boolean
}) => ({
  show: canSave,
  label: '保存配置',
  disabled: !canSave || loading || primaryModelInvalid,
})
