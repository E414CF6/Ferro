"use client";

import React, {createContext, useCallback, useContext, useState} from "react";
import {AlertCircle, CheckCircle2, Info, X} from "lucide-react";

interface Toast {
    id: string;
    message: string;
    type: "success" | "error" | "info";
}

interface ToastContextType {
    showToast: (message: string, type?: "success" | "error" | "info") => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

export function ToastProvider({children}: { children: React.ReactNode }) {
    const [toasts, setToasts] = useState<Toast[]>([]);

    const showToast = useCallback((message: string, type: "success" | "error" | "info" = "info") => {
        const id = Math.random().toString(36).substring(2, 9);
        setToasts((prev) => [...prev, {id, message, type}]);

        setTimeout(() => {
            setToasts((prev) => prev.filter((t) => t.id !== id));
        }, 4000);
    }, []);

    const removeToast = (id: string) => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
    };

    return (
        <ToastContext.Provider value={{showToast}}>
            {children}
            <div className="toast-container">
                {toasts.map((t) => (
                    <div key={t.id} className="toast">
                        {t.type === "success" && <CheckCircle2 size={18} color="#10b981"/>}
                        {t.type === "error" && <AlertCircle size={18} color="#f43f5e"/>}
                        {t.type === "info" && <Info size={18} color="#38bdf8"/>}
                        <span>{t.message}</span>
                        <button
                            onClick={() => removeToast(t.id)}
                            style={{marginLeft: "auto", padding: "2px", opacity: 0.7}}
                        >
                            <X size={14}/>
                        </button>
                    </div>
                ))}
            </div>
        </ToastContext.Provider>
    );
}

export function useToast() {
    const context = useContext(ToastContext);
    if (!context) {
        throw new Error("useToast must be used within a ToastProvider");
    }
    return context;
}
