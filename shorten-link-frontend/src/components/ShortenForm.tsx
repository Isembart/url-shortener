import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function ShortenForm() {
  const [longUrl, setLongUrl] = useState("");
  const [customCode, setCustomCode] = useState("");
  const [shortenedUrl, setShortenedUrl] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    try {
      const response = await fetch("/shorten-link", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: longUrl, code: customCode || undefined })
      });
      
      if (!response.ok) throw new Error("Failed to shorten link");
      
      const data = await response.json();
      setShortenedUrl(data.short_url);
    } catch (error) {
      console.error("Error shortening link:", error);
      alert("Nie udało się skrócić linka!");
    }
    
    setLoading(false);
  };

  return (
    <Card className="max-w-md mx-auto mt-10 p-6 shadow-lg">
      <CardHeader>
        <CardTitle className="text-lg">Skracacz linków</CardTitle>
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
            <a href={"/link/" + shortenedUrl} className="text-blue-500 ml-2" target="_blank" rel="noopener noreferrer">
              {shortenedUrl}
            </a>
          </p>
        )}
      </CardContent>
    </Card>
  );
}
