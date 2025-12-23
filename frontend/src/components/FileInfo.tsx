import { FileText, Database, Columns, File } from "lucide-react";

interface FileInfoProps {
  fileInfo: {
    file_path: string;
    file_size: number;
    total_rows: number;
    total_cols: number;
    headers: string[];
  };
}

function formatFileSize(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
}

export default function FileInfo({ fileInfo }: FileInfoProps) {
  return (
    <div className="bg-gray-800 border-b border-gray-700 px-4 py-3">
      <div className="flex items-center gap-6 flex-wrap">
        <div className="flex items-center gap-2 text-sm">
          <File className="w-4 h-4 text-gray-400" />
          <span className="text-gray-400">文件大小:</span>
          <span className="text-gray-200 font-medium">{formatFileSize(fileInfo.file_size)}</span>
        </div>
        <div className="flex items-center gap-2 text-sm">
          <Database className="w-4 h-4 text-gray-400" />
          <span className="text-gray-400">总行数:</span>
          <span className="text-gray-200 font-medium">
            {fileInfo.total_rows.toLocaleString()}
          </span>
        </div>
        <div className="flex items-center gap-2 text-sm">
          <Columns className="w-4 h-4 text-gray-400" />
          <span className="text-gray-400">总列数:</span>
          <span className="text-gray-200 font-medium">{fileInfo.total_cols}</span>
        </div>
        <div className="flex items-center gap-2 text-sm">
          <FileText className="w-4 h-4 text-gray-400" />
          <span className="text-gray-400">列名:</span>
          <span className="text-gray-200 font-medium">
            {fileInfo.headers.length > 0
              ? fileInfo.headers.slice(0, 5).join(", ") +
                (fileInfo.headers.length > 5 ? "..." : "")
              : "无表头"}
          </span>
        </div>
      </div>
    </div>
  );
}

