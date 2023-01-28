import defaultAxios from "axios";
import { BACKEND_URL } from "./consts";

export const axios = defaultAxios.create({
  baseURL: BACKEND_URL,
  withCredentials: true,
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
