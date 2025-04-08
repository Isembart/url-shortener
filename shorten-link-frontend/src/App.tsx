import LogoutButton from "./components/login/LogoutButton";
import ShortenForm from "./components/ShortenForm";
import {UserLinksList} from "./components/UserLinksList";

function App() {
  return (
    <div className="min-h-screen flex items-center justify-center flex-col bg-gray-100">
      <ShortenForm />
      <UserLinksList/>
      <LogoutButton/>
    </div>
  );
}

export default App;