export async function getVersion() {
  const response = await fetch("/api/version");
  return await response.text();
}

/**
 * Sends a cookie to the server.
 * @param cookie The cookie string to send
 * @returns The fetch response object
 *
 * Possible Status Codes:
 * - 201: Created (Success)
 * - 400: Invalid cookie
 * - 401: Invalid bearer token
 * - 409: Duplicate cookie
 * - 500: Server error
 */
export async function postCookie(cookie: string) {
  const bearer = localStorage.getItem("authToken") || "";
  const response = await fetch("/api/submit", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${bearer}`,
    },
    body: JSON.stringify({ cookie }),
  });

  if (response.status === 400) {
    throw new Error("Invalid cookie format");
  } else if (response.status === 401) {
    throw new Error("Authentication failed. Please set a valid auth token.");
  } else if (response.status === 500) {
    throw new Error("Server error.");
  }

  if (!response.ok) {
    throw new Error(`Error ${response.status}: ${response.statusText}`);
  }
}

/**
 * Gets cookie status information from the server.
 * @returns The cookie status data
 *
 * Possible Status Codes:
 * - 200: Success with cookie status data
 * - 401: Invalid bearer token
 * - 500: Server error
 */
export async function getCookieStatus() {
  const bearer = localStorage.getItem("authToken") || "";
  const response = await fetch("/api/get_cookies", {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${bearer}`,
    },
  });

  if (!response.ok) {
    throw new Error(`Error ${response.status}: ${response.statusText}`);
  }

  return await response.json();
}

/**
 * Deletes a cookie from the server.
 * @param cookie The cookie string to delete
 * @returns The fetch response object
 *
 * Possible Status Codes:
 * - 204: Success (No Content)
 * - 401: Invalid bearer token
 * - 500: Server error
 */
export async function deleteCookie(cookie: string) {
  const bearer = localStorage.getItem("authToken") || "";
  // URL encode the cookie to handle special characters in the URL path
  const encodedCookie = encodeURIComponent(cookie);
  const response = await fetch(`/api/delete_cookie/${encodedCookie}`, {
    method: "DELETE",
    headers: {
      Authorization: `Bearer ${bearer}`,
    },
  });

  return response;
}