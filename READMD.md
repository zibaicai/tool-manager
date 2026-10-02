标准 Conventional Commits 类型：

类型	含义
feat	新功能
fix	修复 bug
docs	文档变更
style	代码格式（不影响逻辑，如空格、分号、缩进）
refactor	重构（既不是新增功能，也不是修 bug）
perf	性能优化
test	测试相关
build	构建系统或外部依赖变更（如 webpack、npm）
ci	CI 配置/脚本变更（如 GitHub Actions、Jenkins）
chore	杂项（不修改 src 或 test 的构建过程、辅助工具等）
revert	回滚某个提交
格式示例：

text
feat: 新增用户登录功能
fix: 修复登录token过期问题
docs: 更新README安装说明
refactor: 重构订单查询逻辑
chore: 升级依赖版本
其他常见扩展类型（团队自定义）：

wip — 开发中（Work In Progress）

release — 发布版本

merge — 合并分支

hotfix — 紧急修复

补充说明：

如果你想在提交时加上范围（scope）和描述，常用格式是：

text
<type>(<scope>): <subject>
例如：fix(auth): 修复token校验失败的问题

另外，很多团队会用工具来强制规范，比如 commitlint + husky，配合 commitizen 交互式生成提交信息。如果你想了解如何配置这套规范，我可以帮你写具体步骤。