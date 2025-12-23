import { useMemo } from "react";
import clsx from "clsx";

interface CSVTableProps {
  headers: string[];
  rows: { fields: string[] }[];
  searchQuery?: string;
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

  if (rows.length === 0) {
    return (
      <div className="text-center py-12 text-gray-400">
        {searchQuery ? "未找到匹配的结果" : "暂无数据"}
      </div>
    );
  }

  return (
    <div className="overflow-x-auto rounded-lg border border-gray-700 bg-gray-800">
      <table className="w-full border-collapse">
        <thead>
          <tr className="bg-gray-750 border-b border-gray-700">
            <th className="sticky left-0 z-10 bg-gray-750 px-4 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider border-r border-gray-700">
              #
            </th>
            {headers.map((header, idx) => (
              <th
                key={idx}
                className="px-4 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider border-r border-gray-700 last:border-r-0"
              >
                {header || `列 ${idx + 1}`}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-700">
          {rows.map((row, rowIdx) => (
            <tr
              key={rowIdx}
              className="hover:bg-gray-750/50 transition-colors"
            >
              <td className="sticky left-0 z-10 bg-gray-800 px-4 py-2 text-sm text-gray-400 border-r border-gray-700 font-mono">
                {rowIdx + 1}
              </td>
              {row.fields.map((field, colIdx) => (
                <td
                  key={colIdx}
                  className="px-4 py-2 text-sm text-gray-300 border-r border-gray-700 last:border-r-0 max-w-xs truncate"
                  title={field}
                >
                  {highlightText(field, searchQuery)}
                </td>
              ))}
              {/* 填充缺失的列 */}
              {row.fields.length < headers.length &&
                Array.from({ length: headers.length - row.fields.length }).map(
                  (_, colIdx) => (
                    <td
                      key={colIdx + row.fields.length}
                      className="px-4 py-2 text-sm text-gray-500 border-r border-gray-700 last:border-r-0"
                    >
                      -
                    </td>
                  )
                )}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

