// 导出工具函数

export interface ExportData {
  headers: string[];
  rows: { fields: string[] }[];
}

/**
 * 将数据导出为 CSV 格式
 */
export function exportToCSV(data: ExportData): string {
  const { headers, rows } = data;
  
  // 创建 CSV 内容
  let csvContent = "";
  
  // 添加表头
  if (headers.length > 0) {
    csvContent += headers.map(escapeCSVField).join(",") + "\n";
  }
  
  // 添加数据行
  rows.forEach((row) => {
    const fields = headers.map((_, idx) => {
      const field = row.fields[idx] || "";
      return escapeCSVField(field);
    });
    csvContent += fields.join(",") + "\n";
  });
  
  return csvContent;
}

/**
 * 将数据导出为 JSON 格式
 */
export function exportToJSON(data: ExportData): string {
  const { headers, rows } = data;
  
  // 转换为对象数组
  const objects = rows.map((row) => {
    const obj: Record<string, string> = {};
    headers.forEach((header, idx) => {
      obj[header || `Column${idx + 1}`] = row.fields[idx] || "";
    });
    return obj;
  });
  
  return JSON.stringify(objects, null, 2);
}

/**
 * 转义 CSV 字段（处理逗号、引号、换行符）
 */
function escapeCSVField(field: string): string {
  if (!field) return "";
  
  // 如果字段包含逗号、引号或换行符，需要用引号包裹
  if (field.includes(",") || field.includes('"') || field.includes("\n") || field.includes("\r")) {
    // 转义引号（" -> ""）
    const escaped = field.replace(/"/g, '""');
    return `"${escaped}"`;
  }
  
  return field;
}

/**
 * 下载文件（使用 Blob 和 URL.createObjectURL）
 */
export function downloadFile(content: string, filename: string, mimeType: string = "text/plain") {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

