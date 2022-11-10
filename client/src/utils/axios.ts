import defaultAxios from "axios";
import { BACKEND_URL } from "./consts";

export const axios = defaultAxios.create({
  baseURL: BACKEND_URL,
  validateStatus: (code) => code >= 200 && code < 500,
});
