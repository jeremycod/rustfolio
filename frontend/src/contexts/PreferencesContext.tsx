import { createContext, useContext, useState, useEffect, ReactNode } from 'react';

interface PreferencesContextType {
  darkMode: boolean;
  toggleDarkMode: () => void;
  notifications: boolean;
  toggleNotifications: () => void;
  autoRefresh: boolean;
  toggleAutoRefresh: () => void;
}

const PreferencesContext = createContext<PreferencesContextType | undefined>(undefined);

export function PreferencesProvider({ children }: { children: ReactNode }) {
  // Load preferences from localStorage
  const [darkMode, setDarkMode] = useState(() => {
    const saved = localStorage.getItem('darkMode');
    return saved ? JSON.parse(saved) : false;
  });

  const [notifications, setNotifications] = useState(() => {
    const saved = localStorage.getItem('notifications');
    return saved ? JSON.parse(saved) : false;
  });

  const [autoRefresh, setAutoRefresh] = useState(() => {
    const saved = localStorage.getItem('autoRefresh');
    return saved ? JSON.parse(saved) : false;
  });

  // Save preferences to localStorage when they change
  useEffect(() => {
    localStorage.setItem('darkMode', JSON.stringify(darkMode));
  }, [darkMode]);

  useEffect(() => {
    localStorage.setItem('notifications', JSON.stringify(notifications));
  }, [notifications]);

  useEffect(() => {
    localStorage.setItem('autoRefresh', JSON.stringify(autoRefresh));
  }, [autoRefresh]);

  // Request notification permission when enabled
  useEffect(() => {
    if (notifications && 'Notification' in window) {
      if (Notification.permission === 'default') {
        Notification.requestPermission();
      }
    }
  }, [notifications]);

  const toggleDarkMode = () => setDarkMode(!darkMode);
  const toggleNotifications = () => setNotifications(!notifications);
  const toggleAutoRefresh = () => setAutoRefresh(!autoRefresh);

  return (
    <PreferencesContext.Provider
      value={{
        darkMode,
        toggleDarkMode,
        notifications,
        toggleNotifications,
        autoRefresh,
        toggleAutoRefresh,
      }}
    >
      {children}
    </PreferencesContext.Provider>
  );
}

export function usePreferences() {
  const context = useContext(PreferencesContext);
  if (context === undefined) {
    throw new Error('usePreferences must be used within a PreferencesProvider');
  }
  return context;
}
