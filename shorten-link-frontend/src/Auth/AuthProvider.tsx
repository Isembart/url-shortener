// import { useQuery } from "@tanstack/react-query";
import LoginForm from "@/components/login/LoginForm";
import { API, API_URL} from "@/utils/api";
import { createContext, useContext, useEffect, useLayoutEffect, useState } from "react"


type AuthContextType = {
    token: string | null,
    setNewToken: (token: String) => void,
}
const AuthContext = createContext<AuthContextType | undefined>(undefined);

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
            if(!(token==="") && !(config as any)._retry) {
                config.headers.Authorization = `${token}`;
                config.withCredentials = true;
            }
            return config;
        })
        
        return () => {
            API.interceptors.request.eject(authInterceptor);
        }
    }, [token]);

    const refreshToken = async () => {
        try{
            const response = await API.get(`${API_URL}/refresh`,{withCredentials: true});
            setToken(response.data.data); 
            return response.data.data;

        } catch{
            setToken("");
        }
    };
    useLayoutEffect(() => {
        const refreshInterceptor = API.interceptors.response.use(
            (response) => response,
            async (error) => {
                const originalRequest = error.config;
                if( error.response.status === 401) {
                    try{
                        const response = await API.get(`${API_URL}/refresh`,{withCredentials: true});
                        originalRequest.headers.Authorization = `${response.data.data}`;
                        originalRequest._retry = true;
                        setToken(response.data.data);
                        return API(originalRequest);
                    } catch {
                        setToken("");
                    }

                } 
            })
        return () => {
            API.interceptors.request.eject(refreshInterceptor);
        }
    }, [token])

    const setNewToken = (newToken: any) => {
        setToken(newToken);
    };

    useEffect(() => {
        refreshToken();
    },[]);

    return(
        <AuthContext.Provider value={{token, setNewToken}}>
            {token!=="" ? children : <LoginForm/>}
        </AuthContext.Provider>
    )
};