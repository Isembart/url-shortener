// import { useQuery } from "@tanstack/react-query";
import API from "@/utils/api";
import { createContext, useContext, useLayoutEffect, useState } from "react"


type AuthContextType = {
    token: string | null,
    setNewToken: (token: String) => void,
}
const AuthContext = createContext<AuthContextType | undefined>(undefined);
const API_URL = import.meta.env.VITE_API_URL || document.URL;

export const useAuth = () => {
    const authContext = useContext(AuthContext);

    if(!authContext){
        throw new Error("useAuth must be used within AuthProvider");
    }

    return authContext;
}

import { ReactNode } from "react";

export const AuthProvider = ({children}: {children: ReactNode}) => {
    const [token, setToken] = useState("");
   
    useLayoutEffect(() => {
        const authInterceptor = API.interceptors.request.use((config) => {
            if(!(token==="") && !config._retry) {
                config.headers.Authorization = `Bearer ${token}`;
            }
            return config;
        })
        
        return () => {
            API.interceptors.request.eject(authInterceptor);
        }
    }, [token])
    
    //response
    useLayoutEffect(() => {
        const refreshInterceptor = API.interceptors.response.use(
            (response) => response,
            async (error) => {
                const originalRequest = error.config;

                if(
                    error.response.status === 403 &&
                    error.response.data.message === 'Unauthorized'
                ) try{
                    console.log("Trying to refresh the token");
                    const response = await API.get(`${API_URL}/api/refreshToken`);
                    setToken(response.data.accessToken);

                    originalRequest.headers.Authorization = `Bearer ${response.data.accessToken}`;
                    originalRequest._retry = true;
                    return API(originalRequest);
                } catch {
                    setToken("");
                }
            })
    })

    const setNewToken = (newToken: any) => {
        setToken(newToken);
    };

    return(
        <AuthContext.Provider value={{token, setNewToken}}>
            {children}
        </AuthContext.Provider>
    )
};