import { create } from 'zustand';

interface StatusBarState {
  isDrawerOpen: boolean;
  activeTab: 'critical' | 'warnings' | 'governance';
  toggleDrawer: () => void;
  openDrawer: () => void;
  closeDrawer: () => void;
  setActiveTab: (tab: 'critical' | 'warnings' | 'governance') => void;
}

export const useStatusBar = create<StatusBarState>((set) => ({
  isDrawerOpen: false,
  activeTab: 'critical',
  toggleDrawer: () => set((state) => ({ isDrawerOpen: !state.isDrawerOpen })),
  openDrawer: () => set({ isDrawerOpen: true }),
  closeDrawer: () => set({ isDrawerOpen: false }),
  setActiveTab: (tab) => set({ activeTab: tab }),
}));
