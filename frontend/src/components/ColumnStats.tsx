import { ColumnStats } from "../utils/columnStats";
import { BarChart3, Hash, Calendar, Type, TrendingUp } from "lucide-react";

interface ColumnStatsProps {
  stats: ColumnStats;
  onClose?: () => void;
}

export default function ColumnStatsPanel({ stats, onClose }: ColumnStatsProps) {
  const formatNumber = (n: number) => {
    if (n % 1 === 0) return n.toLocaleString();
    return n.toFixed(2).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  };

  const getDataTypeIcon = (type: string) => {
    switch (type) {
      case "number":
        return <Hash className="w-4 h-4" />;
      case "date":
        return <Calendar className="w-4 h-4" />;
      case "text":
        return <Type className="w-4 h-4" />;
      default:
        return <BarChart3 className="w-4 h-4" />;
    }
  };

  const getDataTypeColor = (type: string) => {
    switch (type) {
      case "number":
        return "text-blue-400";
      case "date":
        return "text-green-400";
      case "text":
        return "text-yellow-400";
      default:
        return "text-gray-400";
    }
  };

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg p-4 shadow-xl max-w-md">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <BarChart3 className="w-5 h-5 text-primary-500" />
          <h3 className="text-lg font-semibold text-gray-200">
            {stats.columnName || `列 ${stats.columnIndex + 1}`}
          </h3>
        </div>
        {onClose && (
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            ×
          </button>
        )}
      </div>

      {/* 基本信息 */}
      <div className="space-y-3">
        {/* 数据类型 */}
        <div className="flex items-center gap-2 text-sm">
          <span className="text-gray-400">数据类型:</span>
          <div className={`flex items-center gap-1 ${getDataTypeColor(stats.dataType)}`}>
            {getDataTypeIcon(stats.dataType)}
            <span className="capitalize">{stats.dataType}</span>
          </div>
        </div>

        {/* 行数统计 */}
        <div className="grid grid-cols-3 gap-2 text-sm">
          <div>
            <div className="text-gray-400">总行数</div>
            <div className="text-gray-200 font-medium">{stats.totalRows.toLocaleString()}</div>
          </div>
          <div>
            <div className="text-gray-400">非空值</div>
            <div className="text-gray-200 font-medium">{stats.nonNullCount.toLocaleString()}</div>
          </div>
          <div>
            <div className="text-gray-400">空值</div>
            <div className="text-gray-200 font-medium">{stats.nullCount.toLocaleString()}</div>
          </div>
        </div>

        {/* 唯一值 */}
        <div className="text-sm">
          <div className="text-gray-400">唯一值数量</div>
          <div className="text-gray-200 font-medium">{stats.uniqueCount.toLocaleString()}</div>
          <div className="text-xs text-gray-500 mt-1">
            {stats.uniqueCount > 0
              ? `${((stats.uniqueCount / stats.nonNullCount) * 100).toFixed(1)}% 唯一性`
              : ""}
          </div>
        </div>

        {/* 数值统计 */}
        {stats.numericStats && (
          <div className="border-t border-gray-700 pt-3 space-y-2">
            <div className="flex items-center gap-2 text-sm font-semibold text-gray-300">
              <TrendingUp className="w-4 h-4" />
              数值统计
            </div>
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div>
                <div className="text-gray-400">最小值</div>
                <div className="text-gray-200 font-medium">
                  {formatNumber(stats.numericStats.min)}
                </div>
              </div>
              <div>
                <div className="text-gray-400">最大值</div>
                <div className="text-gray-200 font-medium">
                  {formatNumber(stats.numericStats.max)}
                </div>
              </div>
              <div>
                <div className="text-gray-400">平均值</div>
                <div className="text-gray-200 font-medium">
                  {formatNumber(stats.numericStats.mean)}
                </div>
              </div>
              <div>
                <div className="text-gray-400">中位数</div>
                <div className="text-gray-200 font-medium">
                  {formatNumber(stats.numericStats.median)}
                </div>
              </div>
            </div>
            <div className="text-sm">
              <div className="text-gray-400">总和</div>
              <div className="text-gray-200 font-medium">
                {formatNumber(stats.numericStats.sum)}
              </div>
            </div>
          </div>
        )}

        {/* 文本统计 */}
        {stats.textStats && (
          <div className="border-t border-gray-700 pt-3 space-y-2">
            <div className="flex items-center gap-2 text-sm font-semibold text-gray-300">
              <Type className="w-4 h-4" />
              文本统计
            </div>
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div>
                <div className="text-gray-400">最短长度</div>
                <div className="text-gray-200 font-medium">
                  {stats.textStats.minLength} 字符
                </div>
              </div>
              <div>
                <div className="text-gray-400">最长长度</div>
                <div className="text-gray-200 font-medium">
                  {stats.textStats.maxLength} 字符
                </div>
              </div>
              <div>
                <div className="text-gray-400">平均长度</div>
                <div className="text-gray-200 font-medium">
                  {stats.textStats.avgLength.toFixed(1)} 字符
                </div>
              </div>
            </div>
            <div className="text-sm space-y-1">
              <div>
                <div className="text-gray-400">最短值</div>
                <div className="text-gray-200 font-medium truncate" title={stats.textStats.shortestValue}>
                  {stats.textStats.shortestValue}
                </div>
              </div>
              <div>
                <div className="text-gray-400">最长值</div>
                <div className="text-gray-200 font-medium truncate" title={stats.textStats.longestValue}>
                  {stats.textStats.longestValue}
                </div>
              </div>
            </div>
          </div>
        )}

        {/* 日期统计 */}
        {stats.dateStats && (
          <div className="border-t border-gray-700 pt-3 space-y-2">
            <div className="flex items-center gap-2 text-sm font-semibold text-gray-300">
              <Calendar className="w-4 h-4" />
              日期统计
            </div>
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div>
                <div className="text-gray-400">最早日期</div>
                <div className="text-gray-200 font-medium">
                  {stats.dateStats.earliest}
                </div>
              </div>
              <div>
                <div className="text-gray-400">最晚日期</div>
                <div className="text-gray-200 font-medium">
                  {stats.dateStats.latest}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

