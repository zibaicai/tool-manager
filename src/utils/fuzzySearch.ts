/**
 * 模糊搜索纯函数：Damerau-Levenshtein 编辑距离 + 加权评分。
 * 不引第三方库（数据量几十条，O(n*m) DP 无性能压力）。
 *
 * 评分量纲（跨字段比较用，字段权重在外层相乘），四层从高到低：
 *   1000            完全相等
 *   800 - 命中下标   精确子串命中（越靠前越高，下标超过 300 不再衰减）
 *   500 - d*100     整串模糊命中（编辑距离衰减）
 *     + 50          首字符相同（打字手误时开头往往是对的）
 *     + 共同前缀*10  开头连得越长越像
 *     - 长度差*15   输入与候选长度越接近越靠前（权重须高于前缀增益，
 *                   保证 brop→brup 稳压 bro）
 *   450 - dWin*100  窗口模糊命中：query 对齐到长名的任意连续片段
 *                   （自由起止，fzf 同款思路；阈值比整串宽一档，
 *                   修复短 query 带错字找不到长名的边界，如 brop→BurpSuite）
 *   -1              超过阈值，不显示
 */

/** 允许的最大编辑距离：按词长分级（短词严、长词松） */
function maxDist(len: number): number {
  return len <= 4 ? 1 : len <= 8 ? 2 : 3;
}

/**
 * 窗口层阈值：仅在 query 足够长（≥4 字符）时比整串层宽一档。
 * ≤3 字符的 query 不放宽——3 个字符错 2 个的"命中"纯是噪音
 * （如 brp 会错配到 nmap 的 "map" 片段）。
 */
function windowMaxDist(len: number): number {
  return len <= 3 ? 1 : len <= 6 ? 2 : 3;
}

/** Damerau-Levenshtein 编辑距离（含相邻换位，覆盖 brpu→brup 类手误） */
export function damerauDistance(a: string, b: string): number {
  const m = a.length;
  const n = b.length;
  if (!m) return n;
  if (!n) return m;
  const d: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
  for (let i = 0; i <= m; i++) d[i][0] = i;
  for (let j = 0; j <= n; j++) d[0][j] = j;
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + cost);
      if (i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1]) {
        d[i][j] = Math.min(d[i][j], d[i - 2][j - 2] + 1);
      }
    }
  }
  return d[m][n];
}

/**
 * 窗口模糊距离：query 对齐到 text 的任意连续片段的最小编辑距离。
 * 与整串距离的区别：起点自由（第一行全 0）+ 结尾自由（最后一行取最小），
 * 等价于对 text 的所有子串取 min(damerau(q, sub))，但只做一次 O(m*len) DP。
 * 含相邻换位。text 长度小于 query 时退化为整串距离。
 */
function fuzzyWindowDistance(q: string, n: string): number {
  const m = q.length;
  const N = n.length;
  if (!m) return 0;
  if (!N) return m;
  const d: number[][] = Array.from({ length: m + 1 }, () => new Array(N + 1).fill(0));
  for (let i = 0; i <= m; i++) d[i][0] = i;
  // 第一行保持全 0：空 query 前缀可从 n 的任意位置开始对齐（起点自由）
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= N; j++) {
      const cost = q[i - 1] === n[j - 1] ? 0 : 1;
      d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + cost);
      if (i > 1 && j > 1 && q[i - 1] === n[j - 2] && q[i - 2] === n[j - 1]) {
        d[i][j] = Math.min(d[i][j], d[i - 2][j - 2] + 1);
      }
    }
  }
  // 最后一行取最小：对齐终点自由（后缀不必消耗）
  let best = Infinity;
  for (let j = 0; j <= N; j++) best = Math.min(best, d[m][j]);
  return best;
}

/** 单字段相关度评分：负分表示不相关，不显示 */
export function scoreText(query: string, text: string): number {
  const q = query.toLowerCase();
  const n = text.toLowerCase();
  if (!q || !n) return -1;
  if (n === q) return 1000;
  const idx = n.indexOf(q);
  if (idx >= 0) return 800 - Math.min(idx, 300);

  // 整串模糊：整个候选名与 query 的编辑距离
  const d = damerauDistance(q, n);
  if (d <= maxDist(q.length)) {
    let base = 500 - d * 100;
    let prefix = 0;
    while (prefix < q.length && prefix < n.length && q[prefix] === n[prefix]) prefix++;
    if (q[0] === n[0]) base += 50;
    base += prefix * 10;
    base -= Math.abs(q.length - n.length) * 15;
    return base;
  }

  // 窗口模糊：query 带错字时在长名里找近似片段（阈值独立分级，
  // 基线 450 低于整串层，噪音自然沉底）
  const dWin = fuzzyWindowDistance(q, n);
  if (dWin <= windowMaxDist(q.length)) return 450 - dWin * 100;

  return -1;
}

export interface SearchField<T> {
  /** 取字段文本；空串跳过该字段 */
  text: (item: T) => string;
  /** 字段权重（标题 1、描述 0.5 等） */
  weight: number;
}

export interface SearchResult<T> {
  item: T;
  score: number;
}

/**
 * 对 items 逐字段评分取最大值，过滤非正分并按分数降序。
 * limit 控制返回条数（噪音沉底），空查询返回空数组。
 */
export function fuzzySearch<T>(
  query: string,
  items: T[],
  fields: SearchField<T>[],
  limit = 20,
): SearchResult<T>[] {
  const q = query.trim();
  if (!q) return [];
  const out: SearchResult<T>[] = [];
  for (const item of items) {
    let best = -1;
    for (const f of fields) {
      if (f.weight <= 0) continue;
      const text = f.text(item);
      if (!text) continue;
      const s = scoreText(q, text) * f.weight;
      if (s > best) best = s;
    }
    if (best > 0) out.push({ item, score: best });
  }
  out.sort((a, b) => b.score - a.score);
  return out.slice(0, limit);
}
