import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Input } from "../ui/input";
import { Button } from "@/components/ui/button";
import { useMutation } from "@tanstack/react-query";
import API from "@/utils/api";
import { useAuth } from "@/Auth/AuthProvider";


const API_URL = import.meta.env.VITE_API_URL || document.URL;

export default function LoginForm() {
    const [username, setUsername] = useState("");
    const [loginPrompt, setLoginPrompt] = useState("");
    const [password, setPassword] = useState("");
    const Auth = useAuth();


    const loginMutation = useMutation({
        mutationFn: async () => {
            try{
                const response = await API.post(`${API_URL}/login`, {username,password});
                return response.data;
            }catch(error: any) {
                throw new Error(error.response?.data?.error || "Failed to login");
            }
            
            // const response = await fetch(`${API_URL}/login`, {
            //     method: "POST",
            //     headers: { "Content-Type": "application/json"},
            //     body: JSON.stringify({username:username, password:password})
            // });
            // if(!response.ok){
            //     const errorData = await response.json().catch(()=> null);
            //     throw new Error(errorData?.error || "Failed to login");
            // }
            // return response.json();
        },
        onSuccess: (data) => {
            setLoginPrompt("Logged in!");
            Auth.setNewToken(data.token);
        },
        onError: (error) => {
            setLoginPrompt(error.message);
        }
    });

    const registerMutation = useMutation({
        mutationFn: async () => {
            try {
                const response = await API.post(`${API_URL}/create-user`, {
                    username,
                    password
                });
    
                return response.data; // Return only the data, no need to return the full response object
            } catch (error: any) {
                // Extract a meaningful error message
                const errorMessage = error.response?.data?.error || "Failed to create user";
                
                // Throw to trigger React Query’s onError
                throw new Error(errorMessage);
            }
        },
        onSuccess: (data) => {
            setLoginPrompt("Successfully created user!");
            Auth.setNewToken(data.token);
        },
        onError: (error) => {
            setLoginPrompt(error.message);
        }

    });


    return (
        <Card className="max-w-11/12 mx-auto mt-10 py-8 shadow-xl">
            <CardHeader>
                <CardTitle className="text-center text-2xl m-0">Login</CardTitle>
            </CardHeader>
            <CardContent>
                    <Input
                        className="my-2"
                        type="text"
                        placeholder="Username"
                        value={username}
                        onChange={(e)=>setUsername(e.target.value)}
                    />
                    <Input
                        className="my-2"
                        type="password"
                        placeholder="Password"
                        value={password}
                        onChange={(e)=>setPassword(e.target.value)}
                    />

                    <Button onClick={() => {loginMutation.mutate()}} disabled={(registerMutation.isPending || loginMutation.isPending)}> Login </Button>
                    

                    <Button onClick={() => {registerMutation.mutate()}} disabled={(registerMutation.isPending || loginMutation.isPending)}> Create Account </Button>
                    <p>{loginPrompt}</p>
            </CardContent>

        </Card>
    )
}