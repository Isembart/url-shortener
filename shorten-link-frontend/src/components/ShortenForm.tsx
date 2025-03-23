import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

// Get correct backend URL from env
const API_URL = import.meta.env.VITE_API_URL || document.URL;

async function shortenLink(longUrl: string, customCode?: string) {
  const response = await fetch(`${API_URL}/shorten-link`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ url: longUrl, code: customCode || undefined }),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.error || "Failed to shorten link");
  }

  return response.json();
}

export default function ShortenForm() {
  const [longUrl, setLongUrl] = useState("");
  const [customCode, setCustomCode] = useState("");

  const mutation = useMutation({
    mutationFn: () => shortenLink(longUrl, customCode),
    onSuccess: () => {
      setLongUrl("");
      setCustomCode("");
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    mutation.mutate();
  };

  return (
    <Card className="max-w-11/12 mx-auto mt-10 p-1 shadow-xl w-xl">
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
          <Button type="submit" disabled={mutation.isPending} className="w-full">
            {mutation.isPending ? "Przetwarzanie..." : "Skróć link"}
          </Button>
        </form>

        {mutation.isSuccess && (
          <p className="mt-4 text-center">
            Twój skrócony link:{" "}
            <a
              href={`${API_URL}/link/${mutation.data.short_url}`}
              className="text-blue-500 ml-2"
              target="_blank"
              rel="noopener noreferrer"
            >
              {API_URL}/link/{mutation.data.short_url}
            </a>
          </p>
        )}

        {mutation.isError && (
          <p className="mt-4 text-center text-red-500">
            Błąd: {(mutation.error as Error).message}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
