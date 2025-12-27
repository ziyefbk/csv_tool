import { create } from 'zustand';
import { persist } from 'zustand/middleware';

type Theme = 'light' | 'dark' | 'system';

interface UiState {
  theme: Theme;
  sidebarOpen: boolean;
  fontSize: 'sm' | 'base' | 'lg';
  
  // Actions
  setTheme: (theme: Theme) => void;
  toggleSidebar: () => void;
  setFontSize: (size: 'sm' | 'base' | 'lg') => void;
}

export const useUiStore = create<UiState>()(
  persist(
    (set) => ({
      theme: 'system',
      sidebarOpen: true,
      fontSize: 'base',

      setTheme: (theme) => set({ theme }),
      toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
      setFontSize: (size) => set({ fontSize: size }),
    }),
    {
      name: 'ui-storage',
      partialize: (state) => ({ theme: state.theme, fontSize: state.fontSize }),
    }
  )
);

