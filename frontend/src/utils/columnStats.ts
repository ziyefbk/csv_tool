// 列统计信息工具函数

export type DataType = "number" | "text" | "date" | "mixed";

export interface ColumnStats {
  columnIndex: number;
  columnName: string;
  dataType: DataType;
  totalRows: number;
  nonNullCount: number;
  nullCount: number;
  uniqueCount: number;
  
  // 数值统计
  numericStats?: {
    min: number;
    max: number;
    mean: number;
    median: number;
    sum: number;
  };
  
  // 文本统计
  textStats?: {
    minLength: number;
    maxLength: number;
    avgLength: number;
    shortestValue: string;
    longestValue: string;
  };
  
  // 日期统计
  dateStats?: {
    min: Date;
    max: Date;
    earliest: string;
    latest: string;
  };
}

// 检测数据类型
function detectDataType(values: string[]): DataType {
  if (values.length === 0) return "text";
  
  let numberCount = 0;
  let dateCount = 0;
  let textCount = 0;
  
  for (const value of values) {
    if (!value || value.trim() === "") continue;
    
    // 尝试解析为数字
    const num = parseFloat(value);
    if (!isNaN(num) && isFinite(num)) {
      numberCount++;
      continue;
    }
    
    // 尝试解析为日期
    const date = new Date(value);
    if (!isNaN(date.getTime()) && value.match(/^\d{4}-\d{2}-\d{2}/) || value.match(/\d{1,2}\/\d{1,2}\/\d{4}/)) {
      dateCount++;
      continue;
    }
    
    textCount++;
  }
  
  const total = numberCount + dateCount + textCount;
  if (total === 0) return "text";
  
  // 如果超过80%是数字，认为是数字类型
  if (numberCount / total > 0.8) return "number";
  // 如果超过80%是日期，认为是日期类型
  if (dateCount / total > 0.8) return "date";
  // 否则是混合或文本类型
  if (textCount / total > 0.5) return "text";
  return "mixed";
}

// 计算数值统计
function calculateNumericStats(values: string[]): ColumnStats["numericStats"] {
  const numbers = values
    .map(v => parseFloat(v))
    .filter(n => !isNaN(n) && isFinite(n))
    .sort((a, b) => a - b);
  
  if (numbers.length === 0) return undefined;
  
  const sum = numbers.reduce((a, b) => a + b, 0);
  const mean = sum / numbers.length;
  const median = numbers.length % 2 === 0
    ? (numbers[numbers.length / 2 - 1] + numbers[numbers.length / 2]) / 2
    : numbers[Math.floor(numbers.length / 2)];
  
  return {
    min: numbers[0],
    max: numbers[numbers.length - 1],
    mean,
    median,
    sum,
  };
}

// 计算文本统计
function calculateTextStats(values: string[]): ColumnStats["textStats"] {
  const nonEmpty = values.filter(v => v && v.trim() !== "");
  if (nonEmpty.length === 0) return undefined;
  
  const lengths = nonEmpty.map(v => v.length);
  const totalLength = lengths.reduce((a, b) => a + b, 0);
  const avgLength = totalLength / lengths.length;
  
  let shortestValue = nonEmpty[0];
  let longestValue = nonEmpty[0];
  
  for (const value of nonEmpty) {
    if (value.length < shortestValue.length) shortestValue = value;
    if (value.length > longestValue.length) longestValue = value;
  }
  
  return {
    minLength: Math.min(...lengths),
    maxLength: Math.max(...lengths),
    avgLength: Math.round(avgLength * 100) / 100,
    shortestValue,
    longestValue,
  };
}

// 计算日期统计
function calculateDateStats(values: string[]): ColumnStats["dateStats"] {
  const dates = values
    .map(v => new Date(v))
    .filter(d => !isNaN(d.getTime()))
    .sort((a, b) => a.getTime() - b.getTime());
  
  if (dates.length === 0) return undefined;
  
  const min = dates[0];
  const max = dates[dates.length - 1];
  
  return {
    min,
    max,
    earliest: min.toISOString().split("T")[0],
    latest: max.toISOString().split("T")[0],
  };
}

// 计算列统计信息
export function calculateColumnStats(
  columnIndex: number,
  columnName: string,
  rows: { fields: string[] }[]
): ColumnStats {
  const values = rows
    .map(row => row.fields[columnIndex])
    .filter(v => v !== undefined);
  
  const totalRows = rows.length;
  const nonNullCount = values.length;
  const nullCount = totalRows - nonNullCount;
  const uniqueValues = new Set(values);
  const uniqueCount = uniqueValues.size;
  
  const nonEmptyValues = values.filter(v => v && v.trim() !== "");
  const dataType = detectDataType(nonEmptyValues);
  
  const stats: ColumnStats = {
    columnIndex,
    columnName,
    dataType,
    totalRows,
    nonNullCount,
    nullCount,
    uniqueCount,
  };
  
  // 根据数据类型计算相应统计
  if (dataType === "number" || dataType === "mixed") {
    stats.numericStats = calculateNumericStats(nonEmptyValues);
  }
  
  if (dataType === "text" || dataType === "mixed") {
    stats.textStats = calculateTextStats(nonEmptyValues);
  }
  
  if (dataType === "date" || dataType === "mixed") {
    stats.dateStats = calculateDateStats(nonEmptyValues);
  }
  
  return stats;
}

// 计算所有列的统计信息
export function calculateAllColumnStats(
  headers: string[],
  rows: { fields: string[] }[]
): ColumnStats[] {
  return headers.map((header, index) =>
    calculateColumnStats(index, header, rows)
  );
}

