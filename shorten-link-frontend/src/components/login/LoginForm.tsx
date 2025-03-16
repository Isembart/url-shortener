import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Input } from "../ui/input";
import { Button } from "@/components/ui/button";


const API_URL = import.meta.env.VITE_API_URL || document.URL;

export default function LoginForm() {
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");


    const handleLoginForm = async () => {
        const response = await fetch(`${API_URL}/login`, {
            method: "POST",
            headers: { "Content-Type": "application/json"},
            body: JSON.stringify({ username: username, password: password})
        });

        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData.error);
        }
    }

    const handleRegisterForm = async () => {
        const response = await fetch(`${API_URL}/create-user`, {
            method: "POST",
            headers: { "Content-Type": "application/json"},
            body: JSON.stringify({ username: username, password: password})
        });

        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData.error);
        }

    }

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

                    <Button onClick={handleLoginForm}> Login </Button>

                    <Button onClick={handleRegisterForm}> Create Account </Button>
            </CardContent>

        </Card>
    )
}