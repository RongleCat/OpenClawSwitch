import { describe, expect, it } from 'vitest'
import { buildInstallStepIndexMap, buildInstallSteps } from './installSteps'

const labels = {
  check: '环境检测',
  git: '安装 Git',
  node: '安装 Node.js',
  openclaw: '安装 OpenClaw',
  verify: '验证安装',
}

describe('buildInstallSteps', () => {
  it('keeps 5 steps on non-windows', () => {
    const steps = buildInstallSteps(false)
    expect(steps).toHaveLength(5)
    expect(steps.map((step) => step.name)).toEqual([
      labels.check,
      labels.git,
      labels.node,
      labels.openclaw,
      labels.verify,
    ])
  })

  it('keeps Git step and removes legacy NSSM step on windows', () => {
    const steps = buildInstallSteps(true)
    expect(steps).toHaveLength(5)
    expect(steps.map((step) => step.name)).toEqual([
      labels.check,
      labels.git,
      labels.node,
      labels.openclaw,
      labels.verify,
    ])
  })
})

describe('buildInstallStepIndexMap', () => {
  it('maps git/node/openclaw/verify indexes on non-windows', () => {
    const map = buildInstallStepIndexMap(false)
    expect(map.install_git).toBe(1)
    expect(map.install_node).toBe(2)
    expect(map.install_openclaw).toBe(3)
    expect(map.verify).toBe(4)
    expect(map.install_nssm).toBeUndefined()
  })

  it('maps git/node/openclaw/verify indexes on windows', () => {
    const map = buildInstallStepIndexMap(true)
    expect(map.install_git).toBe(1)
    expect(map.install_node).toBe(2)
    expect(map.install_openclaw).toBe(3)
    expect(map.verify).toBe(4)
    expect(map.install_nssm).toBeUndefined()
  })
})
