import ShortenForm from "./components/ShortenForm";
import LoginForm from "./components/login/LoginForm";

function App() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-100">
      <LoginForm />
      <ShortenForm />
    </div>
  );
}

export default App;