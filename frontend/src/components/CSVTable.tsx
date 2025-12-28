import { FixedSizeList } from "react-window";
import { useMemo, useState } from "react";
import { ArrowUp, ArrowDown, Filter, X } from "lucide-react";

interface CSVTableProps {
  headers: string[];
  rows: { fields: string[] }[];
  searchQuery?: string;
  sortColumn?: number | null;
  sortDirection?: "asc" | "desc" | null;
  onSort?: (columnIndex: number | null, direction: "asc" | "desc" | null) => void;
  filters?: Map<number, string>;
  onFilter?: (columnIndex: number, value: string | null) => void;
}

const ROW_HEIGHT = 40; // 每行高度（像素）
const MIN_COLUMN_WIDTH = 100; // 最小列宽
const MAX_COLUMN_WIDTH = 500; // 最大列宽
const DEFAULT_COLUMN_WIDTH = 150; // 默认列宽
const PADDING = 32; // 左右 padding (px-4 = 16px * 2)
const CHAR_WIDTH = 7; // 英文字符平均宽度（像素）
const CJK_CHAR_WIDTH = 14; // 中文字符宽度（像素）

// 估算文本宽度（基于字符数和字符类型）
function estimateTextWidth(text: string): number {
  if (!text) return MIN_COLUMN_WIDTH;
  
  let width = 0;
  for (const char of text) {
    // 检查是否为中日韩字符（包括中文、日文、韩文）
    const code = char.charCodeAt(0);
    if (
      (code >= 0x4e00 && code <= 0x9fff) || // CJK统一汉字
      (code >= 0x3400 && code <= 0x4dbf) || // CJK扩展A
      (code >= 0x20000 && code <= 0x2a6df) || // CJK扩展B
      (code >= 0x3040 && code <= 0x309f) || // 日文平假名
      (code >= 0x30a0 && code <= 0x30ff) || // 日文片假名
      (code >= 0xac00 && code <= 0xd7af)    // 韩文
    ) {
      width += CJK_CHAR_WIDTH;
    } else {
      width += CHAR_WIDTH;
    }
  }
  
  return width;
}

// 智能比较函数（支持数字、日期、文本）
function compareValues(a: string, b: string): number {
  // 尝试数字比较
  const numA = parseFloat(a);
  const numB = parseFloat(b);
  if (!isNaN(numA) && !isNaN(numB)) {
    return numA - numB;
  }
  
  // 尝试日期比较
  const dateA = new Date(a);
  const dateB = new Date(b);
  if (!isNaN(dateA.getTime()) && !isNaN(dateB.getTime())) {
    return dateA.getTime() - dateB.getTime();
  }
  
  // 文本比较
  return a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" });
}

export default function CSVTable({
  headers,
  rows,
  searchQuery = "",
  sortColumn = null,
  sortDirection = null,
  onSort,
  filters = new Map(),
  onFilter,
}: CSVTableProps) {
  const [openFilterMenu, setOpenFilterMenu] = useState<number | null>(null);

  const highlightText = (text: string, query: string) => {
    if (!query) return text;

    const parts = text.split(new RegExp(`(${query})`, "gi"));
    return parts.map((part, i) =>
      part.toLowerCase() === query.toLowerCase() ? (
        <mark key={i} className="bg-yellow-500/50 text-yellow-100">
          {part}
        </mark>
      ) : (
        part
      )
    );
  };

  // 处理排序点击
  const handleSortClick = (columnIndex: number) => {
    if (!onSort) return;
    
    if (sortColumn === columnIndex) {
      // 切换排序方向：asc -> desc -> null
      if (sortDirection === "asc") {
        onSort(columnIndex, "desc");
      } else if (sortDirection === "desc") {
        onSort(null, null);
      } else {
        onSort(columnIndex, "asc");
      }
    } else {
      // 新列，默认升序
      onSort(columnIndex, "asc");
    }
  };

  // 缓存所有列的唯一值（使用 useMemo 避免重复计算）
  const uniqueValuesMap = useMemo(() => {
    const map = new Map<number, string[]>();
    
    headers.forEach((_, colIdx) => {
      const values = new Set<string>();
      rows.forEach((row) => {
        if (
          colIdx < row.fields.length &&
          row.fields[colIdx] !== undefined &&
          row.fields[colIdx] !== null &&
          row.fields[colIdx] !== ""
        ) {
          values.add(row.fields[colIdx]);
        }
      });
      map.set(colIdx, Array.from(values).sort((a, b) => compareValues(a, b)));
    });
    
    return map;
  }, [rows, headers]);

  // 获取列的唯一下拉值（从缓存中获取）
  const getColumnUniqueValues = (columnIndex: number): string[] => {
    return uniqueValuesMap.get(columnIndex) || [];
  };

  // 应用筛选后的行
  const filteredRows = useMemo(() => {
    let result = [...rows];

    // 应用列筛选
    filters.forEach((filterValue, columnIndex) => {
      if (filterValue) {
        result = result.filter((row) => {
          if (columnIndex >= row.fields.length) return false;
          return row.fields[columnIndex] === filterValue;
        });
      }
    });

    return result;
  }, [rows, filters]);

  // 应用排序后的行
  const sortedRows = useMemo(() => {
    if (sortColumn === null || sortDirection === null) {
      return filteredRows;
    }

    const sorted = [...filteredRows];
    sorted.sort((a, b) => {
      const aValue = a.fields[sortColumn] || "";
      const bValue = b.fields[sortColumn] || "";
      const comparison = compareValues(aValue, bValue);
      return sortDirection === "asc" ? comparison : -comparison;
    });

    return sorted;
  }, [filteredRows, sortColumn, sortDirection]);

  // 计算动态列宽
  const finalColumnWidths = useMemo(() => {
    if (sortedRows.length === 0 || headers.length === 0) {
      return headers.map(() => DEFAULT_COLUMN_WIDTH);
    }

    const widths = headers.map((_, colIdx) => {
      // 1. 测量表头宽度（考虑排序和筛选图标）
      const headerText = headers[colIdx] || `列 ${colIdx + 1}`;
      let maxWidth = estimateTextWidth(headerText);
      maxWidth += 60; // 为排序和筛选图标预留空间
      
      // 2. 采样数据行来测量内容宽度
      const sampleSize = Math.min(100, sortedRows.length);
      const step = sortedRows.length > sampleSize ? Math.max(1, Math.floor(sortedRows.length / sampleSize)) : 1;
      
      for (let i = 0; i < sortedRows.length; i += step) {
        const row = sortedRows[i];
        if (row && colIdx < row.fields.length) {
          const field = row.fields[colIdx];
          if (field) {
            const fieldWidth = estimateTextWidth(field);
            maxWidth = Math.max(maxWidth, fieldWidth);
          }
        }
      }
      
      // 3. 添加 padding 并限制在最小/最大范围内
      const width = Math.max(
        MIN_COLUMN_WIDTH,
        Math.min(MAX_COLUMN_WIDTH, maxWidth + PADDING)
      );
      
      return width;
    });

    return widths;
  }, [headers, sortedRows]);

  // 计算容器高度（使用视口高度的 70%，留空间给其他 UI）
  const containerHeight = useMemo(() => {
    const viewportHeight = window.innerHeight;
    const reservedHeight = 300;
    return Math.max(400, viewportHeight - reservedHeight);
  }, []);

  // 计算表格总宽度（行号列 + 数据列）
  const tableWidth = useMemo(() => {
    const rowNumberWidth = 80;
    const dataColumnsWidth = finalColumnWidths.reduce((sum, width) => sum + width, 0);
    return rowNumberWidth + dataColumnsWidth;
  }, [finalColumnWidths]);

  if (rows.length === 0) {
    return (
      <div className="text-center py-12 text-gray-400">
        {searchQuery ? "未找到匹配的结果" : "暂无数据"}
      </div>
    );
  }

  // 虚拟滚动行渲染器
  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const row = sortedRows[index];
    if (!row) return null;

    return (
      <div style={style}>
        <div
          className="flex hover:bg-gray-750/50 transition-colors border-b border-gray-700"
          style={{ width: tableWidth }}
        >
          {/* 行号列 - 固定 */}
          <div className="sticky left-0 z-10 bg-gray-800 px-4 py-2 text-sm text-gray-400 border-r border-gray-700 font-mono flex-shrink-0 flex items-center justify-center w-20">
            {index + 1}
          </div>
          {/* 数据列 */}
          {row.fields.map((field, colIdx) => {
            const width = finalColumnWidths[colIdx] || DEFAULT_COLUMN_WIDTH;
            return (
              <div
                key={colIdx}
                className="px-4 py-2 text-sm text-gray-300 border-r border-gray-700 last:border-r-0 flex-shrink-0 overflow-hidden"
                style={{ width }}
                title={field}
              >
                <div className="truncate">
                  {highlightText(field, searchQuery)}
                </div>
              </div>
            );
          })}
          {/* 填充缺失的列 */}
          {row.fields.length < headers.length &&
            Array.from({ length: headers.length - row.fields.length }).map(
              (_, colIdx) => {
                const width = finalColumnWidths[row.fields.length + colIdx] || DEFAULT_COLUMN_WIDTH;
                return (
                  <div
                    key={colIdx + row.fields.length}
                    className="px-4 py-2 text-sm text-gray-500 border-r border-gray-700 last:border-r-0 flex-shrink-0 flex items-center justify-center"
                    style={{ width }}
                  >
                    -
                  </div>
                );
              }
            )}
        </div>
      </div>
    );
  };

  return (
    <div className="rounded-lg border border-gray-700 bg-gray-800 overflow-hidden">
      {/* 固定表头 */}
      <div className="border-b border-gray-700 bg-gray-750 sticky top-0 z-20 overflow-x-auto">
        <div className="flex" style={{ width: tableWidth }}>
          {/* 行号列头 - 固定 */}
          <div className="sticky left-0 z-30 bg-gray-750 px-4 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider border-r border-gray-700 flex-shrink-0 flex items-center justify-center w-20">
            #
          </div>
          {/* 数据列头 */}
          {headers.map((header, idx) => {
            const width = finalColumnWidths[idx] || DEFAULT_COLUMN_WIDTH;
            const isSorted = sortColumn === idx;
            const hasFilter = filters.has(idx) && filters.get(idx);
            const uniqueValues = getColumnUniqueValues(idx);
            const isFilterMenuOpen = openFilterMenu === idx;

            return (
              <div
                key={idx}
                className="px-4 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider border-r border-gray-700 last:border-r-0 flex-shrink-0 relative group"
                style={{ width }}
              >
                <div className="flex items-center justify-between gap-2">
                  <div
                    className="truncate flex-1 cursor-pointer hover:text-gray-200"
                    onClick={() => handleSortClick(idx)}
                    title={header}
                  >
                    {header || `列 ${idx + 1}`}
                  </div>
                  <div className="flex items-center gap-1 flex-shrink-0">
                    {/* 排序图标 */}
                    {isSorted && (
                      <div
                        className="cursor-pointer text-primary-500"
                        onClick={() => handleSortClick(idx)}
                      >
                        {sortDirection === "asc" ? (
                          <ArrowUp className="w-4 h-4" />
                        ) : (
                          <ArrowDown className="w-4 h-4" />
                        )}
                      </div>
                    )}
                    {/* 筛选按钮 */}
                    <div className="relative">
                      <button
                        onClick={() => setOpenFilterMenu(isFilterMenuOpen ? null : idx)}
                        className={`p-1 rounded hover:bg-gray-600 transition-colors ${
                          hasFilter ? "text-primary-500" : "text-gray-500"
                        }`}
                        title="筛选"
                      >
                        <Filter className="w-4 h-4" />
                      </button>
                      {/* 筛选下拉菜单 */}
                      {isFilterMenuOpen && onFilter && (
                        <>
                          <div
                            className="fixed inset-0 z-40"
                            onClick={() => setOpenFilterMenu(null)}
                          />
                          <div className="absolute top-full left-0 mt-1 bg-gray-700 border border-gray-600 rounded-lg shadow-lg z-50 max-h-64 overflow-y-auto min-w-[200px]">
                            <div className="p-2">
                              <button
                                onClick={() => {
                                  onFilter(idx, null);
                                  setOpenFilterMenu(null);
                                }}
                                className="w-full text-left px-3 py-2 text-sm hover:bg-gray-600 rounded flex items-center justify-between"
                              >
                                <span className={!hasFilter ? "text-primary-400" : ""}>
                                  (全部)
                                </span>
                                {!hasFilter && <X className="w-4 h-4" />}
                              </button>
                              <div className="border-t border-gray-600 my-1" />
                              {uniqueValues.map((value, valueIdx) => {
                                const isSelected = filters.get(idx) === value;
                                return (
                                  <button
                                    key={valueIdx}
                                    onClick={() => {
                                      onFilter(idx, value);
                                      setOpenFilterMenu(null);
                                    }}
                                    className={`w-full text-left px-3 py-2 text-sm hover:bg-gray-600 rounded truncate flex items-center justify-between ${
                                      isSelected ? "text-primary-400" : ""
                                    }`}
                                  >
                                    <span className="truncate">{value}</span>
                                    {isSelected && <X className="w-4 h-4" />}
                                  </button>
                                );
                              })}
                            </div>
                          </div>
                        </>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* 虚拟滚动表格主体 */}
      <div className="overflow-x-auto">
        <FixedSizeList
          height={containerHeight}
          itemCount={sortedRows.length}
          itemSize={ROW_HEIGHT}
          width="100%"
        >
          {Row}
        </FixedSizeList>
      </div>

      {/* 显示总行数 */}
      <div className="px-4 py-2 bg-gray-800 border-t border-gray-700 text-sm text-gray-400">
        共 {sortedRows.length.toLocaleString()} 行
        {rows.length !== sortedRows.length && (
          <span className="ml-2 text-gray-500">
            (已筛选: {rows.length.toLocaleString()} → {sortedRows.length.toLocaleString()})
          </span>
        )}
        {sortedRows.length > 1000 && (
          <span className="ml-2 text-gray-500">
            (使用虚拟滚动优化性能)
          </span>
        )}
      </div>
    </div>
  );
}
