/** 文本包含判断，忽略大小写；keyword 需为已归一化的小写文本。 */
export function includesKeyword(source: string | undefined, keyword: string): boolean {
  return Boolean(source && source.toLowerCase().includes(keyword));
}
