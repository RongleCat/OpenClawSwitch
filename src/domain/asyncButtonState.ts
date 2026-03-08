export interface AsyncButtonStateInput {
  loading: boolean
  baseDisabled?: boolean
}

export interface AsyncButtonLabelInput {
  loading: boolean
  label: string
  loadingLabel: string
}

export interface RunAsyncOnceInput<T> {
  isRunning: () => boolean
  setRunning: (running: boolean) => void
  action: () => Promise<T>
}

export const resolveAsyncButtonState = ({
  loading,
  baseDisabled = false,
}: AsyncButtonStateInput) => ({
  loading,
  disabled: loading || baseDisabled,
})

export const resolveAsyncButtonLabel = ({
  loading,
  label,
  loadingLabel,
}: AsyncButtonLabelInput) => (loading ? loadingLabel : label)

export const runAsyncOnce = async <T>({
  isRunning,
  setRunning,
  action,
}: RunAsyncOnceInput<T>): Promise<T | undefined> => {
  if (isRunning()) {
    return undefined
  }

  setRunning(true)
  try {
    return await action()
  } finally {
    setRunning(false)
  }
}
