import ShortenForm from "./components/ShortenForm";
import LoginForm from "./components/login/LoginForm";

function App() {
  return (
    <div className="min-h-screen flex items-center justify-center flex-col bg-gray-100">
      <ShortenForm />
      <LoginForm />
    </div>
  );
}

export default App;