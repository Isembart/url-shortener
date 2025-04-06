import { API, API_URL } from "@/utils/api";
import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardHeader } from "./ui/card";

interface Link {
    code: string;
    long_url: string;
    short_url: string;
}

const fetchUserLinks = async () => {
    const response = await API.get("/get-user-links");
    // Transform the array of arrays into an array of objects
    return response.data.data.map((item: [string, string]) => ({
        code: item[0],
        long_url: item[1],
        short_url: `${API_URL}/link/${item[0]}`,
    }));
};

export default function UserLinksList() {
    const { data: links, isLoading } = useQuery({
        queryKey: ['userLinks'],
        queryFn: fetchUserLinks,
        refetchOnWindowFocus: false,
    });

    if (isLoading) {
        return <div>Loading...</div>;
    }

    if (!links || links.length === 0) {
        return <div>No links available</div>;
    }

    return (
        <Card className="w-1/2 mt-6 bg-white shadow-xl">
            <CardHeader>
                <h1 className="text-lg text-center">Your links</h1>
            </CardHeader>
            <CardContent>
                <ul>
                    {links.map((link: Link) => (
                        <li key={link.code}>
                            <a href={link.long_url} target="_blank" rel="noopener noreferrer">
                                {link.long_url}
                            </a>
                            <span> - {link.code}</span>
                            <span> - <a href={link.short_url} target="_blank" className="text-blue-500" rel="noopener noreferrer">{link.short_url}</a></span>
                        </li>
                    ))}
                </ul>
            </CardContent>
        </Card>
    );
}