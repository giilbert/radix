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
    throw err;
  }
);
