import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { getMe, login as apiLogin, logout as apiLogout, register as apiRegister, AuthUser } from '../lib/endpoints';

interface AuthContextType {
    user: AuthUser | null;
    login: (email: string, password: string) => Promise<void>;
    logout: () => Promise<void>;
    register: (email: string, password: string, name?: string) => Promise<void>;
    isLoading: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
    const [user, setUser] = useState<AuthUser | null>(null);
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        getMe()
            .then(setUser)
            .catch(() => setUser(null))
            .finally(() => setIsLoading(false));
    }, []);

    const login = async (email: string, password: string) => {
        const user = await apiLogin(email, password);
        setUser(user);
    };

    const logout = async () => {
        await apiLogout();
        setUser(null);
    };

    const register = async (email: string, password: string, name?: string) => {
        await apiRegister(email, password, name);
    };

    return (
        <AuthContext.Provider value={{ user, login, logout, register, isLoading }}>
            {children}
        </AuthContext.Provider>
    );
}

export function useAuth() {
    const ctx = useContext(AuthContext);
    if (!ctx) throw new Error('useAuth must be used within AuthProvider');
    return ctx;
}
