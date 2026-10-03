// 领域类型：结构体以后端 Rust 为唯一来源，ts-rs 生成到 ./bindings.ts（cargo test 时导出）。
// 这里只做字面量联合收窄与再导出，不手写字段，字段增删改由 Rust 侧驱动。
import type { CategoryType, ToolType } from '../constants';
import type {
  Tool as GenTool,
  Category as GenCategory,
  MenuConfig as GenMenuConfig,
} from './bindings';

export type { ToolType, CategoryType };
export type { CustomBg, ThemeSettings } from './bindings';

export type Tool = Omit<GenTool, 'type'> & { type: ToolType };
export type Category = Omit<GenCategory, 'type'> & { type: CategoryType | 'system' };
export type MenuConfig = Omit<GenMenuConfig, 'categories'> & { categories: Category[] };
