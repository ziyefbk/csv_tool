import { FixedSizeList } from "react-window";
import { useMemo } from "react";

interface CSVTableProps {
  headers: string[];
  rows: { fields: string[] }[];
  searchQuery?: string;
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

export default function CSVTable({ headers, rows, searchQuery = "" }: CSVTableProps) {
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

  // 计算动态列宽
  const finalColumnWidths = useMemo(() => {
    if (rows.length === 0 || headers.length === 0) {
      return headers.map(() => DEFAULT_COLUMN_WIDTH);
    }

    const widths = headers.map((_, colIdx) => {
      // 1. 测量表头宽度
      const headerText = headers[colIdx] || `列 ${colIdx + 1}`;
      let maxWidth = estimateTextWidth(headerText);
      
      // 2. 采样数据行来测量内容宽度
      // 采样策略：均匀采样最多100行
      const sampleSize = Math.min(100, rows.length);
      const step = rows.length > sampleSize ? Math.max(1, Math.floor(rows.length / sampleSize)) : 1;
      
      for (let i = 0; i < rows.length; i += step) {
        const row = rows[i];
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
  }, [headers, rows]);

  // 计算容器高度（使用视口高度的 70%，留空间给其他 UI）
  const containerHeight = useMemo(() => {
    const viewportHeight = window.innerHeight;
    // 减去顶部工具栏、文件信息、分页控件的预估高度
    const reservedHeight = 300;
    return Math.max(400, viewportHeight - reservedHeight);
  }, []);

  // 计算表格总宽度（行号列 + 数据列）
  const tableWidth = useMemo(() => {
    const rowNumberWidth = 80; // 行号列宽度
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
    const row = rows[index];
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
            return (
              <div
                key={idx}
                className="px-4 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider border-r border-gray-700 last:border-r-0 flex-shrink-0 overflow-hidden"
                style={{ width }}
                title={header}
              >
                <div className="truncate">{header || `列 ${idx + 1}`}</div>
              </div>
            );
          })}
        </div>
      </div>

      {/* 虚拟滚动表格主体 */}
      <div className="overflow-x-auto">
        <FixedSizeList
          height={containerHeight}
          itemCount={rows.length}
          itemSize={ROW_HEIGHT}
          width="100%"
        >
          {Row}
        </FixedSizeList>
      </div>

      {/* 显示总行数 */}
      <div className="px-4 py-2 bg-gray-800 border-t border-gray-700 text-sm text-gray-400">
        共 {rows.length.toLocaleString()} 行
        {rows.length > 1000 && (
          <span className="ml-2 text-gray-500">
            (使用虚拟滚动优化性能)
          </span>
        )}
      </div>
    </div>
  );
}
