import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

//get correct backend url from env
const API_URL = import.meta.env.VITE_API_URL || document.URL;

export default function ShortenForm() {
  const [longUrl, setLongUrl] = useState("");
  const [customCode, setCustomCode] = useState("");
  const [shortenedUrl, setShortenedUrl] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError("");

    try {
      const response = await fetch(`${API_URL}/shorten-link`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: longUrl, code: customCode || undefined })
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error);
      }
      
      const data = await response.json();
      setShortenedUrl(data.short_url);
    } catch (error: any) {
      console.error("Error shortening link:", error);
      setError(error.message);
    }
    
    setLoading(false);
  };

  return (
    <Card className="max-w-11/12 mx-auto mt-10 p-1 shadow-2xl w-xl">
      <CardHeader>
        <CardTitle className="text-lg text-center">Skracacz linków</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          <Input
            type="url"
            placeholder="Wklej długi link"
            value={longUrl}
            onChange={(e) => setLongUrl(e.target.value)}
            required
          />
          <Input
            type="text"
            placeholder="Opcjonalny kod skróconego linka"
            value={customCode}
            onChange={(e) => setCustomCode(e.target.value)}
          />
          <Button type="submit" disabled={loading} className="w-full">
            {loading ? "Przetwarzanie..." : "Skróć link"}
          </Button>
        </form>
        {shortenedUrl && (
          <p className="mt-4 text-center">
            Twój skrócony link: 
            <a href={API_URL + "/link/" + shortenedUrl} className="text-blue-500 ml-2" target="_blank" rel="noopener noreferrer">
              {API_URL + "/link/" + shortenedUrl}
            </a>
          </p>
        )}
        {error && (
          <p className="mt-4 text-center text-red-500">
            Błąd: {error}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
