import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Input } from "../ui/input";
import { Button } from "@/components/ui/button";
import { useMutation } from "@tanstack/react-query";
import {API, API_URL} from "@/utils/api";
import { useAuth } from "@/Auth/AuthProvider";
import { Checkbox } from "../ui/checkbox";



export default function LoginForm() {
    const [username, setUsername] = useState("");
    const [loginPrompt, setLoginPrompt] = useState("");
    const [persistent, setPersistent] = useState(false);
    const [password, setPassword] = useState("");
    const Auth = useAuth();


    const loginMutation = useMutation({
        mutationFn: async () => {
            try{
                const response = await API.post(`${API_URL}/login`, {username,password, persistent}, {withCredentials:true});
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
            Auth.setNewToken(data.data);
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
                    password,
                    persistent
                });
    
                return response.data; // Return only the data, no need to return the full response object
            } catch (error: any) {
                // Extract a meaningful error message
                const errorMessage = error.response?.data?.error || "Failed to create user";
                
                // Throw to trigger React Query’s onError
                throw new Error(errorMessage);
            }
        },
        onSuccess: () => {
            setLoginPrompt("Successfully created user!");
        },
        onError: (error) => {
            setLoginPrompt(error.message);
        }

    });


    return (
        <Card className="max-w-xl mx-auto mt-10 py-8 shadow-xl">
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
                        required
                    />
                    <Input
                        className="my-2"
                        type="password"
                        placeholder="Password"
                        value={password}
                        onChange={(e)=>setPassword(e.target.value)}
                        required
                    />

                   
                    <div className="items-top flex space-x-2">
                    <Checkbox id="persistent" onCheckedChange={(e:boolean) => {setPersistent(e)}}/>
                        <div className="grid gap-1.5 leading-none">
                            <label htmlFor="persistent" className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                Don't log me out
                            </label>
                        </div>
                    </div>

                    <Button onClick={() => {loginMutation.mutate()}} disabled={(registerMutation.isPending || loginMutation.isPending)}> Login </Button>
                    

                    <Button onClick={() => {registerMutation.mutate()}} disabled={(registerMutation.isPending || loginMutation.isPending)}> Create Account </Button>
                    <p>{loginPrompt}</p>
            </CardContent>

        </Card>
    )
}