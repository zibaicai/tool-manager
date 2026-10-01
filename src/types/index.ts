export type ToolType = 'cmd' | 'exe';

export interface Tool {
  id: string;
  type: ToolType;
  title: string;
  path: string;
  docPath: string | null;
  icon: string | null;
  categoryId: string;
  available: boolean;
  /** EXE 类工具的启动参数（如 -c "..."）；CMD 类无此字段 */
  args?: string | null;
  /** 副标题：工具用途描述，展示在标题下方（EXE 类） */
  desc?: string | null;
  /** 以管理员身份启动，触发 UAC（EXE 类） */
  admin?: boolean;
  /** 关闭脚本（.bat）路径；设置后卡片出现「关闭工具」按钮（EXE 类） */
  stopPath?: string | null;
}

export interface Category {
  id: string;
  name: string;
  type: 'scan' | 'manual' | 'system';
  /** 分类级扫描目录，缺省回退到 MenuConfig.scanRoot */
  scanPath?: string;
  /** 手动归属：汇总目录下划入本分类的一级子目录名；为空则目录为空（工具通过手动分配加入） */
  dirs?: string[];
}

export interface MenuConfig {
  categories: Category[];
  /** 工具目录汇总根目录 */
  scanRoot?: string;
}
