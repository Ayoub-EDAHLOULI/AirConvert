import { HashRouter, Routes, Route } from "react-router-dom";
import Home from "./pages/Home";
import ImagesConverter from "./pages/ImagesConverter";
import AudioConverter from "./pages/AudioConverter";
import DocumentsConverter from "./pages/DocumentsConverter";
import SpreadsheetsConverter from "./pages/SpreadsheetsConverter";
import "./App.css";

function App() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/images" element={<ImagesConverter />} />
        <Route path="/audio" element={<AudioConverter />} />
        <Route path="/documents" element={<DocumentsConverter />} />
        <Route path="/spreadsheets" element={<SpreadsheetsConverter />} />
      </Routes>
    </HashRouter>
  );
}

export default App;
