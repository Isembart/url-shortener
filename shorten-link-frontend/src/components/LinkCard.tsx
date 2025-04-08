import { Card, CardContent, CardHeader } from "./ui/card";
import { LinkData } from "./UserLinksList";

const LinkCard = ({ link }: {link: LinkData}) => {

    return (
        <Card className=" mt-3 py-2 bg-white border-gray-100 mx-auto">
            <CardHeader className="pb-0">
                <h1 className="text-xl text0bik">{link.name || "Untitled"}</h1>
            </CardHeader>
            <CardContent>
                <ul >
                    <li key={link.code} className="mb-2">
                        <a href={link.short_url} target="_blank" className="text-blue-500 hover:underline my-5" rel="noopener noreferrer">
                            {link.short_url}
                        </a>
                    </li>
                    <li>
                        <a href={link.long_url} target="_blank" className="hover:underline" rel="noopener noreferrer">
                            {link.long_url}
                        </a>
                    </li>
                </ul>
            </CardContent>
        </Card>
    );
}

export default LinkCard;