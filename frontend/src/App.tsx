import "./App.css";
import { getVersion } from "./api";
import { useState, useEffect } from "react";
import AuthGatekeeper from "./AuthGatekeeper";
import CookieTabs from "./CookieTabs";

function App() {
  const [version, setVersion] = useState("");
  const [activeTab, setActiveTab] = useState("cookie"); // "cookie" or "token"
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  useEffect(() => {
    // Fetch and set the version when component mounts
    getVersion().then((v) => setVersion(v));

    // Check for authentication status
    const checkAuth = async () => {
      const storedToken = localStorage.getItem("authToken");
      if (storedToken) {
        try {
          const response = await fetch("/api/auth", {
            method: "GET",
            headers: {
              Authorization: `Bearer ${storedToken}`,
              "Content-Type": "application/json",
            },
          });

          if (response.ok) {
            setIsAuthenticated(true);
          } else {
            // Invalid token, clear it
            localStorage.removeItem("authToken");
            setIsAuthenticated(false);
          }
        } catch (error) {
          console.error("Authentication check failed:", error);
          setIsAuthenticated(false);
        }
      }
    };

    checkAuth();
  }, []);

  // Function to handle successful authentication
  const handleAuthenticated = (status: boolean) => {
    setIsAuthenticated(status);
  };

  return (
    <div className="min-h-screen bg-gradient-to-b from-gray-900 to-gray-800 text-white">
      <div className="w-full px-4 sm:px-6 md:px-8 py-10 mx-auto max-w-full sm:max-w-4xl lg:max-w-5xl xl:max-w-6xl">
        <header className="mb-10 text-center">
          <h1 className="text-4xl font-bold mb-2 text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 to-purple-500">
            ClewdR
          </h1>
          <h2 className="text-sm font-mono text-gray-400">{version}</h2>
        </header>

        {isAuthenticated ? (
          // Protected content - only shown when authenticated
          <div className="w-full max-w-md sm:max-w-lg md:max-w-xl mx-auto rounded-xl shadow-xl p-6 border border-gray-700 bg-gray-800/50 backdrop-blur-sm">
            <div className="flex mb-6 border-b border-gray-700">
              <button
                onClick={() => setActiveTab("cookie")}
                className={`flex-1 py-2 font-medium text-sm transition-colors duration-200 ${
                  activeTab === "cookie"
                    ? "text-cyan-400 border-b-2 border-cyan-400"
                    : "text-gray-400 hover:text-gray-300"
                }`}
              >
                Cookie
              </button>
              <button
                onClick={() => setActiveTab("token")}
                className={`flex-1 py-2 font-medium text-sm transition-colors duration-200 ${
                  activeTab === "token"
                    ? "text-purple-400 border-b-2 border-purple-400"
                    : "text-gray-400 hover:text-gray-300"
                }`}
              >
                Auth
              </button>
            </div>

            {activeTab === "cookie" ? (
              <CookieTabs />
            ) : (
              <div className="bg-gray-700 p-6 rounded-lg">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-lg font-medium text-white">Log Out</h3>
                  <button
                    onClick={() => {
                      localStorage.removeItem("authToken");
                      setIsAuthenticated(false);
                    }}
                    className="py-2 px-4 bg-red-600 hover:bg-red-500 text-white rounded-md text-sm font-medium transition-colors duration-200"
                  >
                    Logout
                  </button>
                </div>
                <p className="text-gray-300 text-sm mb-4">
                  You are currently logged in and have access to all features.
                </p>
              </div>
            )}
          </div>
        ) : (
          // Auth gatekeeper - shown when not authenticated
          <div className="w-full max-w-md sm:max-w-lg md:max-w-xl mx-auto rounded-xl shadow-xl overflow-hidden border border-gray-700 bg-gray-800/50 backdrop-blur-sm">
            <div className="p-6">
              <h2 className="text-xl font-semibold text-center mb-6">
                Authentication Required
              </h2>
              <p className="text-gray-400 text-sm mb-6 text-center">
                Please log in with your auth token to access ClewdR features.
              </p>
              <AuthGatekeeper onAuthenticated={handleAuthenticated} />
            </div>
          </div>
        )}

        <footer className="mt-12 text-center text-gray-500 text-sm">
          <p>© {new Date().getFullYear()} ClewdR - All rights reserved</p>
          <div className="mt-2">
            <a
              href="https://github.com/sponsors/Xerxes-2"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1 text-sm font-medium text-pink-400 hover:text-pink-300 transition-colors"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="16"
                height="16"
                fill="currentColor"
                viewBox="0 0 16 16"
                className="inline"
              >
                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z" />
              </svg>
              Buy me a coffee
            </a>
          </div>
        </footer>
      </div>
    </div>
  );
}

export default App;
