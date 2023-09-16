import defaultAxios from "axios";
import { BACKEND_URL } from "./consts";

export const getSessionCookie = () => {
  if (typeof document === "undefined") {
    // TODO: serverside impl
    return "";
  }
  const names = [
    "next-auth.session-token=",
    "__Secure-next-auth.session-token=",
  ];
  const decodedCookie = decodeURIComponent(document.cookie);
  const cookies = decodedCookie.split(";");
  for (const cookie of cookies) {
    if (cookie.startsWith(names[0])) {
      return cookie.substring(names[0].length, cookie.length);
    } else if (cookie.startsWith(names[1])) {
      return cookie.substring(names[1].length, cookie.length);
    }
  }
  return "";
};

export const axios = defaultAxios.create({
  baseURL: BACKEND_URL,
  withCredentials: true,
  headers: {
    Authorization: "Bearer " + getSessionCookie(),
  },
});

axios.interceptors.response.use(
  (res) => res,
  (err) => {
    delete (err as any).stack;
    if (err.response) {
      err.message = (err.response.data as any).error;
      err.code = err.response.statusText;
    } else {
      err.message = "Unable to reach server.";
      err.code = "500";
    }
    throw err;
  }
);
