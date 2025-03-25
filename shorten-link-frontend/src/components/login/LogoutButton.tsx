import { useMutation } from "@tanstack/react-query"
import { Button } from "../ui/button"
import {API, API_URL} from "@/utils/api"
import { useAuth } from "@/Auth/AuthProvider";

export default function LogoutButton() {
    const Auth = useAuth();
    const logoutMutation = useMutation({
        mutationFn: async() => {
            try{
                const response = await API.get(`${API_URL}/logout`, {withCredentials:true});
                return response.data;
            } catch(error) {
                throw error;
            }
        },
        onSuccess: () => {
            
        }
    });

    const logout = () => {
        Auth.setNewToken("");
        logoutMutation.mutate();
    }
   
    return(
        <Button onClick={logout}>
            Logout
        </Button>
    )
}