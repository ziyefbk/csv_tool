import { create } from 'zustand';
import { devtools } from 'zustand/middleware';

export interface CsvFileInfo {
  file_path: string;
  file_size: number;
  total_rows: number;
  total_cols: number;
  headers: string[];
}

interface CsvState {
  currentFile: string | null;
  fileInfo: CsvFileInfo | null;
  currentPage: number;
  pageSize: number;
  searchQuery: string;
  selectedRows: Set<number>;
  
  // Actions
  setCurrentFile: (path: string | null) => void;
  setFileInfo: (info: CsvFileInfo | null) => void;
  setCurrentPage: (page: number) => void;
  setPageSize: (size: number) => void;
  setSearchQuery: (query: string) => void;
  toggleRowSelection: (rowIndex: number) => void;
  clearSelection: () => void;
}

export const useCsvStore = create<CsvState>()(
  devtools(
    (set) => ({
      currentFile: null,
      fileInfo: null,
      currentPage: 0,
      pageSize: 50,
      searchQuery: '',
      selectedRows: new Set(),

      setCurrentFile: (path) => set({ currentFile: path, selectedRows: new Set() }),
      setFileInfo: (info) => set({ fileInfo: info }),
      setCurrentPage: (page) => set({ currentPage: page }),
      setPageSize: (size) => set({ pageSize: size, currentPage: 0 }),
      setSearchQuery: (query) => set({ searchQuery: query, currentPage: 0 }),
      toggleRowSelection: (rowIndex) =>
        set((state) => {
          const newSelection = new Set(state.selectedRows);
          if (newSelection.has(rowIndex)) {
            newSelection.delete(rowIndex);
          } else {
            newSelection.add(rowIndex);
          }
          return { selectedRows: newSelection };
        }),
      clearSelection: () => set({ selectedRows: new Set() }),
    }),
    { name: 'csv-store' }
  )
);

