import React, { useState } from "react";
import "./App.css";
import { postCookie } from "./api";

const CookieSubmitForm = () => {
  const [cookie, setCookie] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [status, setStatus] = useState({
    type: "idle",
    message: "",
  });

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    if (!cookie.trim()) {
      setStatus({
        type: "error",
        message: "Please enter a cookie value",
      });
      return;
    }

    setIsSubmitting(true);
    setStatus({ type: "idle", message: "" });

    try {
      const response = await postCookie(cookie);

      if (response.status === 201 || response.status === 200) {
        setStatus({
          type: "success",
          message: "Cookie submitted successfully!",
        });
        setCookie(""); // Clear the input on success
      } else if (response.status === 400) {
        setStatus({
          type: "error",
          message: "Invalid cookie format",
        });
      } else if (response.status === 401) {
        setStatus({
          type: "error",
          message: "Authentication failed. Please set a valid auth token.",
        });
      } else if (response.status === 409) {
        setStatus({
          type: "error",
          message: "Duplicate cookie – this cookie already exists.",
        });
      } else {
        setStatus({
          type: "error",
          message: `Server error (${response.status})`,
        });
      }
    } catch {
      setStatus({
        type: "error",
        message: "Network error. Please check your connection.",
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div>
      <form onSubmit={handleSubmit} className="space-y-6">
        <div className="space-y-2">
          <label
            htmlFor="cookie"
            className="block text-sm font-medium text-gray-300 mb-1"
          >
            Cookie Value
          </label>
          <div className="relative">
            <textarea
              id="cookie"
              value={cookie}
              onChange={(e) => setCookie(e.target.value)}
              placeholder="Paste your cookie here..."
              className="w-full p-4 bg-gray-700 border border-gray-600 rounded-md focus:ring-2 focus:ring-cyan-500 focus:border-cyan-500 min-h-32 text-sm text-gray-200 shadow-sm transition-all duration-200 placeholder-gray-400"
              disabled={isSubmitting}
            />
            {cookie && (
              <button
                type="button"
                className="absolute top-2 right-2 text-gray-400 hover:text-gray-200"
                onClick={() => setCookie("")}
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-5 w-5"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fillRule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                    clipRule="evenodd"
                  />
                </svg>
              </button>
            )}
          </div>
          <p className="text-xs text-gray-400 mt-1">
            Enter the complete cookie string including all required parameters.
          </p>
        </div>

        {status.message && (
          <div
            className={`p-4 rounded-md flex items-start ${
              status.type === "success"
                ? "bg-green-900 text-green-200 border border-green-700"
                : status.type === "error"
                ? "bg-red-900 text-red-200 border border-red-700"
                : ""
            }`}
          >
            <div className="flex-shrink-0 mr-3">
              {status.type === "success" ? (
                <svg
                  className="h-5 w-5 text-green-300"
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fillRule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                    clipRule="evenodd"
                  />
                </svg>
              ) : (
                <svg
                  className="h-5 w-5 text-red-300"
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fillRule="evenodd"
                    d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                    clipRule="evenodd"
                  />
                </svg>
              )}
            </div>
            <span className="text-sm">{status.message}</span>
          </div>
        )}

        <button
          type="submit"
          disabled={isSubmitting}
          className={`w-full py-3 px-4 rounded-md text-white font-medium transition-all duration-200 ${
            isSubmitting
              ? "bg-cyan-700 cursor-not-allowed"
              : "bg-cyan-600 hover:bg-cyan-500 focus:outline-none focus:ring-2 focus:ring-cyan-500 focus:ring-offset-2 focus:ring-offset-gray-800 shadow-md hover:shadow-lg"
          }`}
        >
          {isSubmitting ? (
            <span className="flex items-center justify-center">
              <svg
                className="animate-spin -ml-1 mr-3 h-5 w-5 text-white"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle
                  className="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  strokeWidth="4"
                ></circle>
                <path
                  className="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                ></path>
              </svg>
              Submitting...
            </span>
          ) : (
            "Submit Cookie"
          )}
        </button>
      </form>
    </div>
  );
};

export default CookieSubmitForm;
