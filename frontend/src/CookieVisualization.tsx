import React, { useState, useEffect } from "react";
import { getCookieStatus, deleteCookie } from "./api";

// Simplified interfaces
interface CookieStatus {
  cookie: string;
  reset_time: number | null;
}

interface UselessCookie {
  cookie: string;
  reason: string | any;
}

interface CookieStatusInfo {
  valid: CookieStatus[];
  dispatched: [CookieStatus, number][];
  exhausted: CookieStatus[];
  invalid: UselessCookie[];
}

// Default empty state
const emptyCookieStatus: CookieStatusInfo = {
  valid: [],
  dispatched: [],
  exhausted: [],
  invalid: [],
};

const CookieVisualization: React.FC = () => {
  const [cookieStatus, setCookieStatus] =
    useState<CookieStatusInfo>(emptyCookieStatus);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [refreshCounter, setRefreshCounter] = useState(0);
  const [expandedCookies, setExpandedCookies] = useState<
    Record<string, boolean>
  >({});
  const [deletingCookie, setDeletingCookie] = useState<string | null>(null);

  // Fetch cookie data
  const fetchCookieStatus = async () => {
    setLoading(true);
    setError(null);

    try {
      const data = await getCookieStatus();
      const safeData: CookieStatusInfo = {
        valid: Array.isArray(data?.valid) ? data.valid : [],
        dispatched: Array.isArray(data?.dispatched) ? data.dispatched : [],
        exhausted: Array.isArray(data?.exhausted) ? data.exhausted : [],
        invalid: Array.isArray(data?.invalid) ? data.invalid : [],
      };
      setCookieStatus(safeData);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setCookieStatus(emptyCookieStatus);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchCookieStatus();
  }, [refreshCounter]);

  const handleRefresh = () => setRefreshCounter((prev) => prev + 1);

  const handleDeleteCookie = async (cookie: string) => {
    if (!window.confirm("Are you sure you want to delete this cookie?")) return;

    setDeletingCookie(cookie);
    setError(null);

    try {
      const response = await deleteCookie(cookie);

      if (response.ok) {
        handleRefresh();
      } else {
        const errorMessage =
          response.status === 401
            ? "Authentication failed. Please provide a valid token."
            : await response
                .json()
                .then(
                  (data) => data.error || `Server error (${response.status})`
                );
        setError(errorMessage);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setDeletingCookie(null);
    }
  };

  // Helper functions
  const formatTimestamp = (timestamp: number): string => {
    if (!timestamp) return "N/A";
    try {
      return new Date(timestamp * 1000).toLocaleString();
    } catch {
      return "Invalid date";
    }
  };

  const formatTimeElapsed = (seconds: number): string => {
    if (!seconds && seconds !== 0) return "unknown";
    if (seconds < 60) return `${seconds} sec`;
    if (seconds < 3600)
      return `${Math.floor(seconds / 60)} min ${seconds % 60} sec`;
    return `${Math.floor(seconds / 3600)} hr ${Math.floor(
      (seconds % 3600) / 60
    )} min`;
  };

  const getReasonText = (reason: any): string => {
    if (!reason) return "Unknown";
    if (typeof reason === "string") return reason;

    try {
      if ("NonPro" in reason) return "Free account";
      if ("Disabled" in reason) return "Organization Disabled";
      if ("Banned" in reason) return "Banned";
      if ("Null" in reason) return "Invalid";
      if ("Restricted" in reason && typeof reason.Restricted === "number")
        return `Restricted until ${formatTimestamp(reason.Restricted)}`;
      if (
        "TooManyRequest" in reason &&
        typeof reason.TooManyRequest === "number"
      )
        return `Rate limited until ${formatTimestamp(reason.TooManyRequest)}`;
    } catch (e) {
      console.error("Error parsing reason:", e, reason);
    }
    return "Unknown";
  };

  const getCookieId = (cookie: string, type: string, index: number): string =>
    `${type}-${index}-${cookie.substring(0, 8)}`;

  const toggleExpand = (cookieId: string) =>
    setExpandedCookies((prev) => ({ ...prev, [cookieId]: !prev[cookieId] }));

  const copyToClipboard = (text: string, event: React.MouseEvent) => {
    event.stopPropagation();
    navigator.clipboard
      .writeText(text)
      .then(() => console.log("Copied to clipboard"))
      .catch((err) => console.error("Failed to copy: ", err));
  };

  // UI Components
  const CookieValue = ({
    cookie,
    cookieId,
  }: {
    cookie: string;
    cookieId: string;
  }) => {
    if (!cookie) return null;
    // Clean cookie value for display
    const cleanCookie = cookie.replace(/sessionKey=sk-ant-sid01-/, "");
    const isExpanded = expandedCookies[cookieId] || false;
    const displayText = isExpanded
      ? cleanCookie
      : `${cleanCookie.substring(0, 30)}${
          cleanCookie.length > 30 ? "..." : ""
        }`;

    return (
      <div className="flex flex-wrap items-center">
        <div
          className="flex items-center cursor-pointer flex-1 mr-2 min-w-0"
          onClick={() => toggleExpand(cookieId)}
        >
          <code
            className={`font-mono ${
              isExpanded ? "break-all" : "truncate"
            } w-full`}
          >
            {displayText}
          </code>
          {cleanCookie.length > 30 && (
            <span className="ml-2 text-gray-500 flex-shrink-0">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d={isExpanded ? "M5 15l7-7 7 7" : "M19 9l-7 7-7-7"}
                />
              </svg>
            </span>
          )}
        </div>
        <button
          onClick={(e) => copyToClipboard(cleanCookie, e)}
          className="p-1 bg-gray-700 hover:bg-gray-600 rounded text-xs text-gray-300 focus:outline-none flex-shrink-0"
          title="Copy to clipboard"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-3 w-3"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3"
            />
          </svg>
        </button>
      </div>
    );
  };

  const DeleteButton = ({ cookie }: { cookie: string }) => (
    <button
      onClick={() => handleDeleteCookie(cookie)}
      disabled={loading || deletingCookie === cookie}
      className={`ml-2 p-1 rounded-md transition-colors ${
        deletingCookie === cookie
          ? "bg-gray-700 text-gray-400 cursor-not-allowed"
          : "text-red-400 hover:text-red-300 hover:bg-red-900/30"
      }`}
      title="Delete cookie"
    >
      {deletingCookie === cookie ? (
        <svg
          className="animate-spin h-4 w-4"
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
      ) : (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          className="h-4 w-4"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
          />
        </svg>
      )}
    </button>
  );

  const CookieSection = ({
    title,
    cookies,
    color,
    renderStatus,
  }: {
    title: string;
    cookies: any[];
    color: string;
    renderStatus: (item: any, index: number) => React.ReactNode;
  }) => (
    <div className={`rounded-lg bg-gray-800 overflow-hidden w-full shadow-md`}>
      <div
        className={`bg-${color}-900 px-4 py-2 flex justify-between items-center border-b border-${color}-700`}
      >
        <h4 className={`font-medium text-${color}-100`}>{title}</h4>
        <span
          className={`bg-${color}-800 text-${color}-100 text-xs px-2 py-1 rounded-full`}
        >
          {cookies.length}
        </span>
      </div>
      {cookies.length > 0 ? (
        <div className="p-4 divide-y divide-gray-700">
          {cookies.map((item, index) => renderStatus(item, index))}
        </div>
      ) : (
        <div className="p-4 text-sm text-gray-400 italic">
          No {title.toLowerCase()}
        </div>
      )}
    </div>
  );

  // Calculate total cookie count
  const totalCookies =
    cookieStatus.valid.length +
    cookieStatus.dispatched.length +
    cookieStatus.exhausted.length +
    cookieStatus.invalid.length;

  return (
    <div className="space-y-6 w-full">
      {/* Header */}
      <div className="flex justify-between items-center mb-4 w-full">
        <div>
          <h3 className="text-lg font-semibold text-white">Cookie Status</h3>
          <p className="text-xs text-gray-400 mt-1">
            Total: {totalCookies} cookies
          </p>
        </div>
        <button
          onClick={handleRefresh}
          className="p-2 bg-gray-700 hover:bg-gray-600 rounded-md transition-colors text-sm"
          disabled={loading}
        >
          {loading ? (
            <span className="flex items-center">
              <svg
                className="animate-spin h-4 w-4 mr-2"
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
              Refreshing...
            </span>
          ) : (
            <span className="flex items-center">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-4 w-4 mr-2"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                />
              </svg>
              Refresh
            </span>
          )}
        </button>
      </div>

      {/* Error Display */}
      {error && (
        <div className="p-4 bg-red-900 text-red-200 border border-red-700 rounded-md">
          <p className="text-sm">{error}</p>
        </div>
      )}

      {/* Loading State */}
      {loading && totalCookies === 0 && (
        <div className="flex justify-center py-8">
          <svg
            className="animate-spin h-8 w-8 text-cyan-500"
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
        </div>
      )}

      {/* Cookie Sections */}
      <div className="space-y-6 w-full">
        {/* Valid Cookies */}
        <CookieSection
          title="Valid Cookies"
          cookies={cookieStatus.valid}
          color="green"
          renderStatus={(status, index) => {
            const cookieId = getCookieId(status.cookie, "valid", index);
            return (
              <div
                key={index}
                className="py-2 text-sm text-gray-300 flex flex-wrap justify-between items-start"
              >
                <div className="text-green-300 flex-grow mr-4 min-w-0 mb-1 sm:mb-0">
                  <CookieValue cookie={status.cookie} cookieId={cookieId} />
                </div>
                <div className="flex items-center">
                  <span className="text-gray-400">Available</span>
                  <DeleteButton cookie={status.cookie} />
                </div>
              </div>
            );
          }}
        />

        {/* Dispatched Cookies */}
        <CookieSection
          title="In-Use Cookies"
          cookies={cookieStatus.dispatched}
          color="blue"
          renderStatus={(item, index) => {
            const [status, time] = item;
            const cookieId = getCookieId(status.cookie, "dispatched", index);
            return (
              <div
                key={index}
                className="py-2 flex flex-wrap justify-between text-sm items-start"
              >
                <div className="text-blue-300 flex-grow mr-4 min-w-0 mb-1 sm:mb-0">
                  <CookieValue cookie={status.cookie} cookieId={cookieId} />
                </div>
                <div className="flex items-center">
                  <span className="text-gray-400">
                    Used for {formatTimeElapsed(time)}
                  </span>
                  <DeleteButton cookie={status.cookie} />
                </div>
              </div>
            );
          }}
        />

        {/* Exhausted Cookies */}
        <CookieSection
          title="Exhausted Cookies"
          cookies={cookieStatus.exhausted}
          color="yellow"
          renderStatus={(status, index) => {
            const cookieId = getCookieId(status.cookie, "exhausted", index);
            return (
              <div
                key={index}
                className="py-2 flex flex-wrap justify-between text-sm items-start"
              >
                <div className="text-yellow-300 flex-grow mr-4 min-w-0 mb-1 sm:mb-0">
                  <CookieValue cookie={status.cookie} cookieId={cookieId} />
                </div>
                <div className="flex items-center">
                  <span className="text-gray-400">
                    {status.reset_time
                      ? `Resets at ${formatTimestamp(status.reset_time)}`
                      : "Unknown reset time"}
                  </span>
                  <DeleteButton cookie={status.cookie} />
                </div>
              </div>
            );
          }}
        />

        {/* Invalid Cookies */}
        <CookieSection
          title="Invalid Cookies"
          cookies={cookieStatus.invalid}
          color="red"
          renderStatus={(status, index) => {
            const cookieId = getCookieId(status.cookie, "invalid", index);
            return (
              <div
                key={index}
                className="py-2 flex flex-wrap justify-between text-sm items-start"
              >
                <div className="text-red-300 flex-grow mr-4 min-w-0 mb-1 sm:mb-0">
                  <CookieValue cookie={status.cookie} cookieId={cookieId} />
                </div>
                <div className="flex items-center">
                  <span className="text-gray-400">
                    {getReasonText(status.reason)}
                  </span>
                  <DeleteButton cookie={status.cookie} />
                </div>
              </div>
            );
          }}
        />
      </div>
    </div>
  );
};

export default CookieVisualization;
