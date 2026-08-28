"use client";

import React, {createContext, useCallback, useContext, useEffect, useState} from "react";
import {User} from "./types";
import {fetchGraphQL, MUTATIONS, QUERIES} from "./graphql";

interface AuthContextType {
    user: User | null;
    token: string | null;
    loading: boolean;
    login: (usernameOrEmail: string, password: string) => Promise<void>;
    signup: (data: {
        username: string;
        email: string;
        password: string;
        displayName: string;
        bio?: string;
        avatarUrl?: string;
        headerImageUrl?: string;
        location?: string;
        website?: string;
    }) => Promise<void>;
    logout: () => void;
    refreshUser: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({children}: { children: React.ReactNode }) {
    const [user, setUser] = useState<User | null>(null);
    const [token, setToken] = useState<string | null>(null);
    const [loading, setLoading] = useState<boolean>(true);

    const fetchCurrentUser = useCallback(async (authToken: string) => {
        try {
            const data = await fetchGraphQL<{ me: User }>(QUERIES.ME, {}, authToken);
            if (data && data.me) {
                setUser(data.me);
            } else {
                localStorage.removeItem("ferro_token");
                setToken(null);
                setUser(null);
            }
        } catch (err) {
            console.error("Failed to fetch current user:", err);
            localStorage.removeItem("ferro_token");
            setToken(null);
            setUser(null);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        const savedToken = localStorage.getItem("ferro_token");
        if (savedToken) {
            setToken(savedToken);
            fetchCurrentUser(savedToken);
        } else {
            setLoading(false);
        }
    }, [fetchCurrentUser]);

    const login = async (usernameOrEmail: string, password: string) => {
        const data = await fetchGraphQL<{ login: { token: string; user: User } }>(
            MUTATIONS.LOGIN,
            {usernameOrEmail, password}
        );
        if (data?.login?.token) {
            localStorage.setItem("ferro_token", data.login.token);
            setToken(data.login.token);
            setUser(data.login.user);
        }
    };

    const signup = async (formData: {
        username: string;
        email: string;
        password: string;
        displayName: string;
        bio?: string;
        avatarUrl?: string;
        headerImageUrl?: string;
        location?: string;
        website?: string;
    }) => {
        const data = await fetchGraphQL<{ signup: { token: string; user: User } }>(
            MUTATIONS.SIGNUP,
            formData
        );
        if (data?.signup?.token) {
            localStorage.setItem("ferro_token", data.signup.token);
            setToken(data.signup.token);
            setUser(data.signup.user);
        }
    };

    const logout = () => {
        localStorage.removeItem("ferro_token");
        setToken(null);
        setUser(null);
    };

    const refreshUser = async () => {
        if (token) {
            await fetchCurrentUser(token);
        }
    };

    return (
        <AuthContext.Provider
            value={{
                user,
                token,
                loading,
                login,
                signup,
                logout,
                refreshUser,
            }}
        >
            {children}
        </AuthContext.Provider>
    );
}

export function useAuth() {
    const context = useContext(AuthContext);
    if (!context) {
        throw new Error("useAuth must be used within an AuthProvider");
    }
    return context;
}
