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
            Auth.setNewToken("");
        }
    });

    const logout = () => {
        logoutMutation.mutate();
    }
   
    return(
        <Button onClick={logout} className="w-1/10 m-5 bg-white border-none absolute top-0 right-0 hover:bg-gray-200">
            Logout
        </Button>
    )
}