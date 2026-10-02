# Tool Manager

基于 **Tauri 2 + Vue 3 + TypeScript** 的本地工具箱管理器，用于统一管理和启动渗透测试/日常工作中的 CMD 命令行工具与 EXE/GUI 程序。

- 前端：Vue 3 + Pinia + Vite
- 后端：Rust（Tauri 2）
- 平台：Windows（窗口与提权逻辑使用 Win32 API）

---

## 一、快速开始

### 环境要求

- Node.js 18+
- Rust 工具链（stable）
- Windows 10/11

### 安装与开发

```bash
npm install          # 安装前端依赖，并自动执行 husky 安装 Git 钩子
npm run tauri dev    # 启动开发模式（热更新前端）
```

### 仅构建前端

```bash
npm run build        # vue-tsc 类型检查 + vite 打包到 dist/
```

### 发布构建（重要）

必须使用 Tauri 打包命令，**不要**直接用 `cargo build --release`（后者会连接 localhost:1420 开发服务器，无 dev server 时白屏）：

```bash
# 1. 结束正在运行的应用
# 2. 构建 release 产物（不生成安装包）
npx tauri build --no-bundle
# 3. 用 src-tauri/target/release/tool-manager.exe 覆盖安装目录中的同名文件
# 4. 重新启动应用
```

---

## 二、工具功能

### 1. CMD 类工具（命令行工具）

- **自动扫描**：按 `menu.json` 中配置的 `scanRoot` 扫描各工具目录
- **分类展示**：每个分类可用 `dirs` 白名单指定归属的工具文件夹；`dirs` 为空的新分类初始为空
- **孤儿兜底**：未被任何分类收录、也未手动分配的工具，默认显示在第一个 CMD 分类下，不会丢失
- **手动分配**：工具卡片 → 编辑 → "所属目录"，可把任意工具分配到指定 CMD 分类（工具文件不移动）；选择"自动"恢复扫描归属
- **卡片操作**：
  - 命令窗口：在可见的 cmd 窗口中执行该工具的启动命令（窗口保留不闪退）
  - 使用文档：渲染工具目录下的 `README.md` / `Ops.md`（Markdown + 代码高亮）
  - 打开目录：在资源管理器中打开工具文件夹
- **自定义标题/副标题**：编辑后存入覆盖配置，空标题恢复为默认（README H1 或目录名）
- **注意**：移动或重命名工具文件夹会导致手动分配/编辑覆盖失效，需重新编辑

### 2. EXE 类工具（GUI / 服务类程序）

- **添加方式**：在对应 EXE 分类页点击右上角"＋ 添加 EXE 工具"，粘贴完整路径或命令，例如：
  - `D:\Program\Goby\Goby.exe`
  - `C:\Windows\System32\net.exe start "Tenable Nessus"`
  - `D:\scripts\start-nessus.bat`
- 自动按第一个 `.exe` / `.bat` 拆分为 `exePath` + `args`（带引号参数原样保留）
- **控制台识别**：自动读取 PE 头判断控制台/GUI 程序，控制台程序（net.exe、bat/cmd 脚本）以可见 `cmd /K` 窗口启动，输出可见不闪退
- **以管理员身份运行**：勾选后经 UAC（`runas`）提权启动，适用于 `net start/stop` 等需要系统服务权限的命令
- **关闭脚本**：可为工具配置一个 `.bat` 关闭脚本（`stopPath`），卡片上出现"关闭工具"按钮，用于停止服务类程序；同样支持管理员提权
- 支持编辑标题、副标题描述
- 每个 EXE 工具通过 `categoryId` 归属到各自独立分类

### 3. 分类（目录）管理

- 侧栏左下角按钮：新增 / 重命名 / 删除分类
- 分类分两类：
  - `scan`：CMD 扫描分类，可配置 `dirs` 工具白名单
  - `manual`：EXE 分类，工具手动添加
- 删除分类有确认弹窗，显示目录名、类型和工具数量；包含 EXE 工具时有警告
- 修改 `menu.json` 后点击应用内"刷新"即时生效
- 约束：已有分类的 `id` 不可随意更改（exe-tools.json 依赖它做关联）

### 4. 主题与外观

- 默认 / 浅色 / 深色三种模式
- 可自定义背景图（铺满窗口）
- 页面透明度滑杆：**0%～100%**（0% 为全透明背景，注意可读性）
- 卡片、按钮透明度按页面透明度阶梯联动
- 无边框窗口 + 自绘标题栏：
  - 按住标题栏拖动窗口，双击最大化/还原
  - 最小化 / 最大化 / 关闭按钮
  - 四边四角可拖拽缩放，最大化自动避让任务栏
  - 窗口去除系统灰边与顶部白线，保留 Win11 原生圆角和阴影

---

## 三、配置文件与优先级

配置文件按以下**三级优先级**查找（命中即用）：

1. **便携模式**：exe 同目录 `config/menu.json`（绿色版 / U 盘）
2. **开发模式**：debug 构建时使用项目树内 `config/`
3. **用户级目录**：`%APPDATA%\com.toolmanager.app\`（首次运行从项目 config/ 迁移）

| 文件 | 作用 | 编码 |
|---|---|---|
| `menu.json` | scanRoot 与分类列表（id/name/type/dirs） | UTF-8 无 BOM |
| `exe-tools.json` | EXE 工具列表（路径、参数、分类、提权、关闭脚本） | UTF-8 无 BOM |
| `cmd-tools.json` | CMD 工具覆盖项（标题/副标题、手动分配的 categoryId） | UTF-8 无 BOM |
| 主题设置 | 背景图、模式、透明度（经后端命令持久化） | — |

> JSON 配置必须以 **UTF-8 无 BOM** 保存，否则 serde_json 解析失败。
> PowerShell 脚本（如 update.ps1 / deploy.ps1）需保存为 **UTF-8 BOM**，否则 Windows PowerShell 5.1 会按 GBK 读取导致中文报错。

### exe-tools.json 工具字段说明

```json
{
  "id": "441addd92bb60488",
  "exePath": "C:\\Windows\\System32\\net.exe",
  "args": "start \"Tenable Nessus\"",
  "categoryId": "exe-host-scan",
  "admin": true,
  "stopPath": "D:\\scripts\\stop-nessus.bat",
  "title": "Nessus",
  "desc": "漏洞扫描服务"
}
```

### menu.json 分类字段说明

```json
{
  "scanRoot": "D:\\Py Scripting tool\\cmd_tool",
  "categories": [
    { "id": "web-scan", "name": "Web 扫描", "type": "scan", "dirs": ["JSFinder-master"] },
    { "id": "exe-other", "name": "其他", "type": "manual" }
  ]
}
```

---

## 四、代码提交规范（Commitlint + Husky）

项目已集成 **Conventional Commits** 提交规范，提交信息不合法会被 `commit-msg` 钩子拦截。

### 提交格式

```
<type>(scope?): <subject>
```

- `type`：提交类型（必填，小写）
- `scope`：影响范围（可选）
- `subject`：简明描述（必填）

### type 取值

| type | 用途 |
|---|---|
| `feat` | 新功能 |
| `fix` | 修复缺陷 |
| `docs` | 仅文档变更 |
| `style` | 代码格式（不影响逻辑，如空格、分号） |
| `refactor` | 重构（非新增功能、非修 bug） |
| `perf` | 性能优化 |
| `test` | 新增/修改测试 |
| `build` | 构建系统或依赖变更（Cargo、vite、npm） |
| `ci` | CI 配置变更 |
| `chore` | 杂项（配置、脚本等） |
| `revert` | 回滚提交 |

### 示例

```bash
git commit -m "feat(launcher): EXE工具支持关闭脚本停止服务"
git commit -m "fix(window): 去除无边框窗口顶部白色高光线"
git commit -m "docs: 补充README工具功能与提交规范"
git commit -m "refactor(scanner): CMD分类孤儿工具兜底逻辑"
git commit -m "chore(deps): 升级tauri到2.1"
```

### 规则要点

- subject 不能为空，建议用中文清晰描述
- 建议正文说明「为什么改」，而非只写「改了什么」
- 紧急情况可临时跳过钩子（不推荐）：

```bash
git commit --no-verify -m "hotfix: 临时修复"
```

### 钩子机制

- `.husky/commit-msg`：提交时执行 `npx --no-install commitlint --edit`
- `package.json` 的 `prepare: "husky"`：`npm install` 后自动安装钩子
- 新克隆仓库执行一次 `npm install` 即可生效；配置见 `commitlint.config.js`

---

## 五、常见问题

- **点击 EXE 工具窗口闪退**：控制台程序已用 `cmd /K` 保留窗口；若是服务启动失败，查看窗口中的报错（常见为"系统错误 5 拒绝访问"，需勾选"以管理员身份运行"）
- **CMD 工具在分类里消失**：检查是否被分配到其他分类；未归属的工具会兜底显示在第一个 CMD 分类
- **改了配置不生效**：点击应用内"刷新"按钮
- **release 版本白屏**：确认是用 `npx tauri build --no-bundle` 构建而非 `cargo build --release`
