import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {API, API_URL} from "@/utils/api";





export default function ShortenForm() {
  const [longUrl, setLongUrl] = useState("");
  const [customCode, setCustomCode] = useState("");
  const qc = useQueryClient();

  const shortenLinkMutation = useMutation({
    mutationFn: async () => {
      try{
        const response = await API.post(`${API_URL}/shorten-link`, {url:longUrl, code: (customCode || undefined)}, {withCredentials:true})
        return response.data;
      }catch(err){
        console.log(err);
        throw err;
      }
    },
    onSuccess: (data) => {
      // setLongUrl("");
      // setCustomCode("");
      console.log("data: ", data);
      qc.invalidateQueries({ queryKey: ["userLinks"] });
    },
    onError: (error) => {
    console.log("error: ", error);  
    }
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    shortenLinkMutation.mutate();
  };

  return (
    <Card className="max-w-11/12 mx-auto mt-10 p-1 shadow-xl w-xl bg-white border-0 text-center">
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
          <Button type="submit" disabled={shortenLinkMutation.isPending} className="w-1/3 border-1 m-2 hover:bg-gray-200">
            {shortenLinkMutation.isPending ? "Przetwarzanie..." : "Skróć link"}
          </Button>
        </form>

        {shortenLinkMutation.isSuccess && (
          <p className="mt-4 text-center">
            Twój skrócony link:{" "}
            <a
              href={`${API_URL}/link/${shortenLinkMutation.data.data}`}
              className="text-blue-500 ml-2"
              target="_blank"
              rel="noopener noreferrer"
            >
              {API_URL}/link/{shortenLinkMutation.data.data}
            </a>
          </p>
        )}

        {shortenLinkMutation.isError && (
          <p className="mt-4 text-center text-red-500">
            Błąd: {(shortenLinkMutation.error as Error).message}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
