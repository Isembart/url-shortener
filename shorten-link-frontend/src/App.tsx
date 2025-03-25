import LogoutButton from "./components/login/LogoutButton";
import ShortenForm from "./components/ShortenForm";

function App() {
  return (
    <div className="min-h-screen flex items-center justify-center flex-col bg-gray-100">
      <ShortenForm />
      <LogoutButton/>
    </div>
  );
}

export default App;