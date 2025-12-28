import { useState, useMemo, useEffect, useRef } from "react";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { FileText, Loader2, Search, X } from "lucide-react";
import CSVTable from "./components/CSVTable";
import FileInfo from "./components/FileInfo";
import Pagination from "./components/Pagination";

interface CsvFileInfo {
  file_path: string;
  file_size: number;
  total_rows: number;
  total_cols: number;
  headers: string[];
}

interface CsvRow {
  fields: string[];
}

interface PageData {
  rows: CsvRow[];
  page: number;
  total_pages: number;
  page_size: number;
}

function App() {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [fileInfo, setFileInfo] = useState<CsvFileInfo | null>(null);
  const [pageData, setPageData] = useState<PageData | null>(null);
  const [currentPage, setCurrentPage] = useState(0);
  const [pageSize, setPageSize] = useState(200); // Increased for better virtual scrolling performance
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [debouncedSearchQuery, setDebouncedSearchQuery] = useState("");
  const [sortColumn, setSortColumn] = useState<number | null>(null);
  const [sortDirection, setSortDirection] = useState<"asc" | "desc" | null>(null);
  const [filters, setFilters] = useState<Map<number, string>>(new Map());
  const searchTimeoutRef = useRef<number | null>(null);

  // 搜索防抖：延迟 300ms 执行搜索
  useEffect(() => {
    if (searchTimeoutRef.current) {
      clearTimeout(searchTimeoutRef.current);
    }
    
    searchTimeoutRef.current = window.setTimeout(() => {
      setDebouncedSearchQuery(searchQuery);
    }, 300);

    return () => {
      if (searchTimeoutRef.current !== null) {
        window.clearTimeout(searchTimeoutRef.current);
      }
    };
  }, [searchQuery]);

  const handleOpenFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "CSV Files",
            extensions: ["csv"],
          },
        ],
      });

      if (selected && typeof selected === "string") {
        setLoading(true);
        setError(null);
        setFilePath(selected);
        setCurrentPage(0);
        setSearchQuery("");
        setDebouncedSearchQuery("");
        setSortColumn(null);
        setSortDirection(null);
        setFilters(new Map());

        try {
          const info = await invoke<CsvFileInfo>("open_csv_file", {
            filePath: selected,
            hasHeaders: true,
            delimiter: null,
            indexGranularity: null,
          });

          setFileInfo(info);
          await loadPage(selected, 0, pageSize);
        } catch (err) {
          setError(err instanceof Error ? err.message : "打开文件失败");
        } finally {
          setLoading(false);
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "选择文件失败");
    }
  };

  const loadPage = async (path: string, page: number, size: number) => {
    try {
      setLoading(true);
      const data = await invoke<PageData>("read_page", {
        filePath: path,
        page,
        pageSize: size,
      });
      setPageData(data);
      setCurrentPage(page);
    } catch (err) {
      setError(err instanceof Error ? err.message : "加载页面失败");
    } finally {
      setLoading(false);
    }
  };

  const handlePageChange = (newPage: number) => {
    if (filePath && pageData && newPage >= 0 && newPage < pageData.total_pages) {
      loadPage(filePath, newPage, pageSize);
    }
  };

  const handlePageSizeChange = (newSize: number) => {
    setPageSize(newSize);
    if (filePath) {
      loadPage(filePath, 0, newSize);
    }
  };

  const handleSort = (columnIndex: number | null, direction: "asc" | "desc" | null) => {
    setSortColumn(columnIndex);
    setSortDirection(direction);
  };

  const handleFilter = (columnIndex: number, value: string | null) => {
    const newFilters = new Map(filters);
    if (value) {
      newFilters.set(columnIndex, value);
    } else {
      newFilters.delete(columnIndex);
    }
    setFilters(newFilters);
  };

  // 应用全局搜索筛选（使用 useMemo 缓存结果，避免不必要的重新计算）
  // 使用防抖后的搜索查询，减少频繁计算
  const filteredRows = useMemo(() => {
    if (!pageData?.rows) return [];
    if (!debouncedSearchQuery) return pageData.rows;
    
    const query = debouncedSearchQuery.toLowerCase();
    return pageData.rows.filter((row) =>
      row.fields.some((field) => field.toLowerCase().includes(query))
    );
  }, [pageData?.rows, debouncedSearchQuery]);

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-white">
      {/* 顶部工具栏 */}
      <div className="bg-gray-800 border-b border-gray-700 px-4 py-3 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <button
            onClick={handleOpenFile}
            className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors"
          >
            <FileText className="w-5 h-5" />
            打开CSV文件
          </button>
          {filePath && (
            <div className="flex items-center gap-2 text-sm text-gray-400">
              <FileText className="w-4 h-4" />
              <span className="max-w-md truncate">{filePath}</span>
            </div>
          )}
        </div>

        {fileInfo && (
          <div className="flex items-center gap-4">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
              <input
                type="text"
                placeholder="搜索..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10 pr-8 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 w-64"
              />
              {searchQuery && (
                <button
                  onClick={() => setSearchQuery("")}
                  className="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white"
                >
                  <X className="w-4 h-4" />
                </button>
              )}
            </div>
          </div>
        )}
      </div>

      {/* 主要内容区域 */}
      <div className="flex-1 overflow-hidden flex flex-col">
        {error && (
          <div className="bg-red-900/50 border border-red-700 text-red-200 px-4 py-3 mx-4 mt-4 rounded-lg">
            {error}
          </div>
        )}

        {!fileInfo ? (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <FileText className="w-24 h-24 mx-auto mb-4 text-gray-600" />
              <h2 className="text-2xl font-semibold mb-2 text-gray-300">
                欢迎使用 CSV Tool
              </h2>
              <p className="text-gray-500 mb-6">
                点击上方按钮打开CSV文件开始查看
              </p>
              <button
                onClick={handleOpenFile}
                className="px-6 py-3 bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors"
              >
                打开文件
              </button>
            </div>
          </div>
        ) : (
          <>
            {/* 文件信息 */}
            <FileInfo fileInfo={fileInfo} />

            {/* 表格区域 */}
            <div className="flex-1 overflow-auto px-4 pb-4">
              {loading ? (
                <div className="flex items-center justify-center h-full">
                  <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
                </div>
              ) : (
                <CSVTable
                  headers={fileInfo.headers}
                  rows={debouncedSearchQuery ? filteredRows : (pageData?.rows || [])}
                  searchQuery={debouncedSearchQuery}
                  sortColumn={sortColumn}
                  sortDirection={sortDirection}
                  onSort={handleSort}
                  filters={filters}
                  onFilter={handleFilter}
                />
              )}
            </div>

            {/* 分页控件 */}
            {pageData && (
              <div className="border-t border-gray-700 px-4 py-3 bg-gray-800">
                <Pagination
                  currentPage={currentPage}
                  totalPages={pageData.total_pages}
                  pageSize={pageSize}
                  onPageChange={handlePageChange}
                  onPageSizeChange={handlePageSizeChange}
                />
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

export default App;

