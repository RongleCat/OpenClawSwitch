# 测试规范

## 1. 测试架构

### 1.1 测试框架

| 组件 | 框架 | 版本 |
|------|------|------|
| 前端测试 | Vitest | 4.0.x |
| 测试环境 | jsdom | 25.0.x |
| 断言库 | Vitest expect | 内置 |
| Mock 工具 | Vitest vi | 内置 |

### 1.2 测试目录结构

```
openclaw-manager-tauri/
├── tests/                    # 集成测试
│   ├── tauriConfig.test.ts   # Tauri 配置测试
│   ├── setupConfig.test.ts   # 配置测试
│   ├── windowSpec.test.ts    # 窗口规格测试
│   └── ...
│
├── src/
│   ├── domain/               # 业务逻辑
│   │   ├── *.ts              # 源代码
│   │   └── *.test.ts         # 单元测试
│   ├── components/           # 组件
│   │   ├── *.tsx             # 源代码
│   │   └── *.test.tsx        # 组件测试
│   └── lib/                  # 工具库
│       ├── *.ts              # 源代码
│       └── *.test.ts         # 工具测试
│
└── src-tauri/
    └── src/                  # Rust 后端
        ├── *.rs              # 源代码
        └── tests/            # Rust 测试
```

---

## 2. 测试类型

### 2.1 单元测试 (Unit Tests)

**目的**：测试单个函数、方法或组件的行为

**位置**：与被测文件同目录，`.test.ts` 后缀

**示例**：
```typescript
// src/domain/installSteps.ts
export function buildInstallSteps(isWindows: boolean) {
  // ...
}

// src/domain/installSteps.test.ts
import { describe, expect, it } from 'vitest';
import { buildInstallSteps } from './installSteps';

describe('buildInstallSteps', () => {
  it('keeps 5 steps on non-windows', () => {
    const steps = buildInstallSteps(false);
    expect(steps).toHaveLength(5);
  });

  it('keeps Git step on windows', () => {
    const steps = buildInstallSteps(true);
    expect(steps).toHaveLength(5);
  });
});
```

### 2.2 集成测试 (Integration Tests)

**目的**：测试多个模块之间的交互

**位置**：`tests/` 目录

**示例**：
```typescript
// tests/tauriConfig.test.ts
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("tauri build commands", () => {
  it("keeps the base config aligned with bundled runtime resources", () => {
    const tauriConfig = readJson("src-tauri/tauri.conf.json");

    expect(tauriConfig.build.beforeDevCommand)
      .not.toContain("npm run");
    expect(tauriConfig.bundle?.resources)
      .toContain("resources/vendor/openclaw");
  });
});
```

### 2.3 组件测试 (Component Tests)

**目的**：测试 React 组件的渲染和交互

**示例**：
```tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { Button } from '@/components/ui/button';

describe('Button', () => {
  it('renders with default variant', () => {
    render(<Button>Click</Button>);
    expect(screen.getByRole('button')).toHaveClass('bg-primary');
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>Click</Button>);

    fireEvent.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });
});
```

---

## 3. 测试约定

### 3.1 文件命名

```
✅ 推荐
- installSteps.test.ts      # 与源文件同名
- tauriConfig.test.ts       # 集成测试

❌ 不推荐
- installSteps_test.ts
- test-installSteps.ts
- installSteps.spec.ts      # 统一使用 .test.ts
```

### 3.2 测试结构

```typescript
import { describe, expect, it, beforeEach, vi } from 'vitest';

describe('模块名', () => {
  // 钩子函数
  beforeEach(() => {
    // 每个测试前的设置
  });

  // 测试函数
  describe('函数名/功能组', () => {
    it('应该... (具体行为描述)', () => {
      // 测试代码
    });

    it('应该... (边界条件)', () => {
      // 测试代码
    });
  });
});
```

### 3.3 断言风格

```typescript
// ✅ 推荐：具体明确的断言
expect(result).toBe(42);
expect(array).toHaveLength(5);
expect(object).toHaveProperty('name');
expect(string).toContain('substring');

// ❌ 不推荐：模糊的断言
expect(result).toBeTruthy();  // 太宽泛
expect(result).toEqual({});   // 可能忽略细节
```

---

## 4. Mock 和 Stub

### 4.1 函数 Mock

```typescript
import { vi } from 'vitest';

// Mock 函数
const mockFn = vi.fn();
mockFn.mockReturnValue(42);
mockFn.mockResolvedValue({ data: 'test' });
mockFn.mockRejectedValue(new Error('error'));

// 使用
result = mockFn('arg');

// 断言
expect(mockFn).toHaveBeenCalledWith('arg');
expect(mockFn).toHaveBeenCalledTimes(1);
```

### 4.2 模块 Mock

```typescript
import { vi } from 'vitest';

// Mock 整个模块
vi.mock('@/lib/desktop', () => ({
  invokeSafe: vi.fn(),
  // ...
}));

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));
```

### 4.3 计时器 Mock

```typescript
import { vi } from 'vitest';

vi.useFakeTimers();

// 推进时间
vi.advanceTimersByTime(1000);

// 运行所有待处理定时器
vi.runAllTimers();

// 恢复真实定时器
vi.useRealTimers();
```

---

## 5. 测试覆盖率

### 5.1 运行覆盖率

```bash
# 生成覆盖率报告
pnpm test -- --coverage

# 覆盖率输出格式
# - text (终端)
# - html (浏览器)
# - lcov (CI 集成)
```

### 5.2 覆盖率目标

| 指标 | 目标 | 说明 |
|------|------|------|
| 语句覆盖率 | ≥80% | 执行的语句比例 |
| 分支覆盖率 | ≥70% | 执行的分支比例 |
| 函数覆盖率 | ≥85% | 调用的函数比例 |
| 行覆盖率 | ≥80% | 执行的行比例 |

### 5.3 覆盖率配置

```typescript
// vite.config.ts
export default defineConfig({
  test: {
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src-tauri/',
        '**/*.d.ts',
        '**/*.config.*',
      ],
    },
  },
});
```

---

## 6. 测试最佳实践

### 6.1 AAA 模式

```typescript
import { describe, expect, it } from 'vitest';
import { add } from './math';

describe('add', () => {
  it('应该返回两个数的和', () => {
    // Arrange - 准备
    const a = 2;
    const b = 3;

    // Act - 执行
    const result = add(a, b);

    // Assert - 断言
    expect(result).toBe(5);
  });
});
```

### 6.2 测试命名

```typescript
// ✅ 推荐：描述性命名
it('应该返回空数组当输入为空', () => {});
it('应该抛出错误当参数无效', () => {});
it('应该忽略大小写进行比较', () => {});

// ❌ 不推荐：模糊命名
it('测试基本功能', () => {});
it('工作正常', () => {});
it('测试 1', () => {});
```

### 6.3 独立测试

```typescript
// ✅ 推荐：每个测试独立
it('应该处理有效输入', () => {
  const result = process('input');
  expect(result).toBe('output');
});

it('应该处理无效输入', () => {
  const result = process('');
  expect(result).toBeNull();
});

// ❌ 不推荐：测试间依赖
it('应该设置状态', () => {
  state.value = 42;
  expect(state.value).toBe(42);
});

it('应该使用之前的状态', () => {
  // 依赖上一个测试的状态
  expect(state.value).toBe(42);
});
```

### 6.4 测试边界条件

```typescript
describe('parseNumber', () => {
  // 正常情况
  it('应该解析有效数字', () => {
    expect(parseNumber('42')).toBe(42);
  });

  // 边界值
  it('应该处理零', () => {
    expect(parseNumber('0')).toBe(0);
  });

  it('应该处理负数', () => {
    expect(parseNumber('-42')).toBe(-42);
  });

  // 异常情况
  it('应该抛出错误当输入不是数字', () => {
    expect(() => parseNumber('abc')).toThrow();
  });

  it('应该抛出错误当输入为空', () => {
    expect(() => parseNumber('')).toThrow();
  });
});
```

---

## 7. Rust 测试

### 7.1 单元测试

```rust
// src-tauri/src/main.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_open_path_command_uses_textedit_on_macos() {
        let (program, args) =
            build_open_path_command_for_os(Path::new("/tmp/openclaw.json"), "macos")
                .unwrap();
        assert_eq!(program, "open");
        assert_eq!(args, vec!["-a", "TextEdit", "/tmp/openclaw.json"]);
    }

    #[test]
    fn build_open_path_command_uses_notepad_on_windows() {
        let (program, args) =
            build_open_path_command_for_os(Path::new(r"C:\temp\openclaw.json"), "windows")
                .unwrap();
        assert_eq!(program, "notepad.exe");
        assert_eq!(args, vec![r"C:\temp\openclaw.json"]);
    }
}
```

### 7.2 运行 Rust 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 显示输出
cargo test -- --nocapture

# 单线程运行
cargo test -- --test-threads=1
```

---

## 8. 持续集成

### 8.1 GitHub Actions 配置

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Setup pnpm
        uses: pnpm/action-setup@v2
        with:
          version: 9

      - name: Install dependencies
        run: pnpm install

      - name: Run tests
        run: pnpm test

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### 8.2 测试报告

**HTML 报告**：
```bash
pnpm test -- --coverage --reporter=html
# 打开 coverage/index.html
```

**JUnit 格式**：
```bash
pnpm test -- --reporter=junit --outputFile=results.xml
```

---

## 9. 常见测试模式

### 9.1 异步代码测试

```typescript
//  async/await
it('应该异步获取数据', async () => {
  const data = await fetchData();
  expect(data).toHaveProperty('items');
});

// Promise
it('应该解决 Promise', () => {
  return expect(Promise.resolve(42)).resolves.toBe(42);
});

// 错误处理
it('应该拒绝无效的输入', async () => {
  await expect(fetchData(null)).rejects.toThrow();
});
```

### 9.2 Hook 测试

```typescript
import { renderHook, act } from '@testing-library/react';
import { useCounter } from './useCounter';

describe('useCounter', () => {
  it('应该初始化为 0', () => {
    const { result } = renderHook(() => useCounter());
    expect(result.current.count).toBe(0);
  });

  it('应该增加计数', () => {
    const { result } = renderHook(() => useCounter());

    act(() => {
      result.current.increment();
    });

    expect(result.current.count).toBe(1);
  });
});
```

### 9.3 快照测试

```typescript
import { render } from '@testing-library/react';
import { Card } from '@/components/ui/card';

it('应该渲染正确的快照', () => {
  const { container } = render(
    <Card>
      <Card.Header>标题</Card.Header>
      <Card.Content>内容</Card.Content>
    </Card>
  );

  expect(container).toMatchSnapshot();
});
```

**更新快照**：
```bash
pnpm test -- -u
```

---

## 10. 测试检查清单

### 10.1 代码提交前

- [ ] 所有测试通过
- [ ] 覆盖率没有显著下降
- [ ] 新代码有对应测试
- [ ] 测试命名清晰描述行为

### 10.2 代码审查时

- [ ] 测试覆盖边界条件
- [ ] 测试独立，无相互依赖
- [ ] Mock 使用合理
- [ ] 断言具体明确

### 10.3 发布前

- [ ] 完整测试套件通过
- [ ] 集成测试通过
- [ ] 性能测试通过
- [ ] 手动测试关键路径

---

*最后更新：2026-03-19*
