import { invoke } from '@tauri-apps/api/tauri';
import type { CsvFileInfo } from '@/stores/csvStore';

// 重新导出类型以便在其他地方使用
export type { CsvFileInfo };

export interface CsvRow {
  fields: string[];
}

export interface PageData {
  rows: CsvRow[];
  page: number;
  total_pages: number;
  page_size: number;
}

export interface SearchOptions {
  pattern: string;
  regex?: boolean;
  ignoreCase?: boolean;
  column?: string;
  maxResults?: number;
}

export interface SearchResult {
  row_number: number;
  matches: Array<{ column: number; field: string }>;
  record: CsvRow;
}

/**
 * Quick preview result - available immediately without waiting for index
 */
export interface QuickPreview {
  headers: string[];
  rows: CsvRow[];
  file_size: number;
  estimated_rows: number;
  is_complete: boolean;
}

export const csvApi = {
  /**
   * 打开CSV文件
   */
  openFile: async (
    filePath: string,
    options?: {
      hasHeaders?: boolean;
      delimiter?: string;
      indexGranularity?: number;
    }
  ): Promise<CsvFileInfo> => {
    return invoke<CsvFileInfo>('open_csv_file', {
      filePath,
      hasHeaders: options?.hasHeaders ?? true,
      delimiter: options?.delimiter ?? null,
      indexGranularity: options?.indexGranularity ?? null,
    });
  },

  /**
   * 读取指定页的数据
   */
  readPage: async (
    filePath: string,
    page: number,
    pageSize: number
  ): Promise<PageData> => {
    return invoke<PageData>('read_page', {
      filePath,
      page,
      pageSize,
    });
  },

  /**
   * 搜索CSV文件
   */
  search: async (
    filePath: string,
    options: SearchOptions
  ): Promise<SearchResult[]> => {
    return invoke<SearchResult[]>('search_csv', {
      filePath,
      ...options,
    });
  },

  /**
   * 关闭文件
   */
  closeFile: async (filePath: string): Promise<void> => {
    return invoke('close_file', { filePath });
  },

  /**
   * 获取文件信息
   */
  getFileInfo: async (filePath: string): Promise<CsvFileInfo> => {
    return invoke<CsvFileInfo>('get_file_info', { filePath });
  },

  /**
   * Quick preview - instantly shows first N rows without waiting for index
   * Use this for large files to provide immediate feedback
   */
  quickPreview: async (
    filePath: string,
    previewRows: number = 100,
    delimiter?: string
  ): Promise<QuickPreview> => {
    return invoke<QuickPreview>('quick_preview', {
      filePath,
      previewRows,
      delimiter: delimiter ?? null,
    });
  },
};

