import defaultAxios from "axios";
import { BACKEND_URL } from "./consts";

export const getSessionCookie = () => {
  if (typeof document === "undefined") {
    // TODO: serverside impl
    return "";
  }
  const name = "next-auth.session-token=";
  const decodedCookie = decodeURIComponent(document.cookie);
  const cookies = decodedCookie.split(";");
  for (const cookie of cookies) {
    if (cookie.indexOf(name) === 0) {
      return cookie.substring(name.length, cookie.length);
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
