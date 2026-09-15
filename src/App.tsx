import { HashRouter, Routes, Route } from "react-router-dom";
import Home from "./pages/Home";
import ImagesConverter from "./pages/ImagesConverter";
import "./App.css";

function App() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/images" element={<ImagesConverter />} />
      </Routes>
    </HashRouter>
  );
}

export default App;
